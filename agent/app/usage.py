"""Normalize LangChain/LangGraph token metadata for the desktop usage ledger."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

USAGE_SSE_SCHEMA_VERSION = 1


@dataclass(slots=True)
class UsageMeasurement:
    run_id: str
    model: str | None = None
    input_tokens: int | None = None
    output_tokens: int | None = None
    total_tokens: int | None = None
    cache_read_tokens: int | None = None
    cache_creation_tokens: int | None = None
    reasoning_tokens: int | None = None
    source: str = "provider_reported"
    provider_metadata: dict[str, Any] = field(default_factory=dict)

    def completeness(self) -> int:
        return sum(
            value is not None
            for value in (
                self.input_tokens,
                self.output_tokens,
                self.total_tokens,
                self.cache_read_tokens,
                self.cache_creation_tokens,
                self.reasoning_tokens,
            )
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "model": self.model,
            "input_tokens": self.input_tokens,
            "output_tokens": self.output_tokens,
            "total_tokens": self.total_tokens,
            "cache_read_tokens": self.cache_read_tokens,
            "cache_creation_tokens": self.cache_creation_tokens,
            "reasoning_tokens": self.reasoning_tokens,
            "source": self.source,
            "provider_metadata": self.provider_metadata,
        }


class UsageCollector:
    """Upsert one most-complete measurement per model run_id."""

    def __init__(self, default_model: str | None = None) -> None:
        self._default_model = default_model
        self._measurements: dict[str, UsageMeasurement] = {}

    def observe(self, event: dict[str, Any]) -> None:
        kind = str(event.get("event") or "")
        if kind not in {
            "on_chat_model_stream",
            "on_chat_model_end",
            "on_llm_stream",
            "on_llm_end",
        }:
            return
        run_id = str(event.get("run_id") or event.get("id") or "").strip()
        if not run_id:
            return

        data = event.get("data") or {}
        candidates = [
            _get(data, "chunk"),
            _get(data, "output"),
            data,
        ]
        usage, usage_keys = _best_usage(candidates)
        if not usage:
            return

        model = _extract_model(event, candidates) or self._default_model
        incoming = UsageMeasurement(
            run_id=run_id,
            model=model,
            input_tokens=_token(usage, "input_tokens", "prompt_tokens"),
            output_tokens=_token(usage, "output_tokens", "completion_tokens"),
            total_tokens=_token(usage, "total_tokens"),
            cache_read_tokens=_token(
                usage,
                "cache_read_tokens",
                "cached_input_tokens",
                "cache_read_input_tokens",
            ),
            cache_creation_tokens=_token(
                usage,
                "cache_creation_tokens",
                "cache_creation_input_tokens",
                "cache_write_tokens",
            ),
            reasoning_tokens=_token(usage, "reasoning_tokens"),
            provider_metadata={"raw_usage_keys": sorted(usage_keys)},
        )
        if incoming.completeness() == 0:
            return
        current = self._measurements.get(run_id)
        self._measurements[run_id] = _merge(current, incoming)

    def event_payload(self) -> dict[str, Any] | None:
        if not self._measurements:
            return None
        return {
            "schema_version": USAGE_SSE_SCHEMA_VERSION,
            "measurements": [
                measurement.to_dict()
                for measurement in self._measurements.values()
            ],
        }


def _merge(
    current: UsageMeasurement | None,
    incoming: UsageMeasurement,
) -> UsageMeasurement:
    if current is None:
        return incoming
    # End events commonly add totals/cache fields. Merge field-wise rather than
    # summing: stream and end are two views of the same run, not two calls.
    for name in (
        "input_tokens",
        "output_tokens",
        "total_tokens",
        "cache_read_tokens",
        "cache_creation_tokens",
        "reasoning_tokens",
    ):
        value = getattr(incoming, name)
        if value is not None:
            setattr(current, name, value)
    if incoming.model:
        current.model = incoming.model
    keys = set(current.provider_metadata.get("raw_usage_keys", []))
    keys.update(incoming.provider_metadata.get("raw_usage_keys", []))
    current.provider_metadata["raw_usage_keys"] = sorted(keys)
    return current


def _best_usage(candidates: list[Any]) -> tuple[dict[str, Any], set[str]]:
    best: dict[str, Any] = {}
    best_keys: set[str] = set()
    for candidate in candidates:
        for mapping in _usage_mappings(candidate):
            keys = set(mapping)
            score = sum(
                _token(mapping, key) is not None
                for key in (
                    "input_tokens",
                    "prompt_tokens",
                    "output_tokens",
                    "completion_tokens",
                    "total_tokens",
                    "cache_read_tokens",
                    "cached_input_tokens",
                    "cache_creation_tokens",
                    "reasoning_tokens",
                )
            )
            if score > sum(value is not None for value in best.values()):
                best = mapping
                best_keys = keys
            elif score > 0:
                for key, value in mapping.items():
                    if value is not None:
                        best[key] = value
                best_keys.update(keys)
    return best, best_keys


def _usage_mappings(value: Any) -> list[dict[str, Any]]:
    if value is None:
        return []
    mappings: list[dict[str, Any]] = []
    for key in ("usage_metadata", "usage", "token_usage"):
        raw = _get(value, key)
        mapping = _as_mapping(raw)
        if mapping:
            mappings.append(mapping)
    response_metadata = _get(value, "response_metadata")
    if response_metadata:
        for key in ("usage", "token_usage"):
            mapping = _as_mapping(_get(response_metadata, key))
            if mapping:
                mappings.append(mapping)
    llm_output = _get(value, "llm_output")
    if llm_output:
        for key in ("usage", "token_usage"):
            mapping = _as_mapping(_get(llm_output, key))
            if mapping:
                mappings.append(mapping)
    if isinstance(value, (list, tuple)):
        for item in value:
            mappings.extend(_usage_mappings(item))
    return mappings


def _extract_model(event: dict[str, Any], candidates: list[Any]) -> str | None:
    metadata = event.get("metadata") or {}
    for value in (
        _get(metadata, "ls_model_name"),
        _get(metadata, "model"),
        *(
            candidate_model
            for candidate in candidates
            for candidate_model in (
                _get(candidate, "model"),
                _get(candidate, "model_name"),
                _get(_get(candidate, "response_metadata"), "model_name"),
                _get(_get(candidate, "response_metadata"), "model"),
            )
        ),
    ):
        if value:
            return str(value)
    return None


def _token(mapping: dict[str, Any], *keys: str) -> int | None:
    for key in keys:
        value = mapping.get(key)
        if value is None:
            continue
        try:
            token = int(value)
        except (TypeError, ValueError):
            continue
        if token >= 0:
            return token
    return None


def _get(value: Any, key: str) -> Any:
    if value is None:
        return None
    if isinstance(value, dict):
        return value.get(key)
    return getattr(value, key, None)


def _as_mapping(value: Any) -> dict[str, Any]:
    if isinstance(value, dict):
        return value
    if value is None:
        return {}
    if hasattr(value, "model_dump"):
        dumped = value.model_dump()
        return dumped if isinstance(dumped, dict) else {}
    return {
        key: getattr(value, key)
        for key in dir(value)
        if not key.startswith("_") and not callable(getattr(value, key, None))
    }
