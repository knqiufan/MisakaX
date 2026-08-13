from types import SimpleNamespace

from app.usage import UsageCollector


def test_collector_dedupes_stream_and_end_by_run_id() -> None:
    collector = UsageCollector(default_model="fallback")
    collector.observe(
        {
            "event": "on_chat_model_stream",
            "run_id": "run-1",
            "data": {"chunk": SimpleNamespace(usage_metadata={"input_tokens": 10})},
        }
    )
    collector.observe(
        {
            "event": "on_chat_model_end",
            "run_id": "run-1",
            "metadata": {"ls_model_name": "actual-model"},
            "data": {
                "output": SimpleNamespace(
                    usage_metadata={
                        "input_tokens": 10,
                        "output_tokens": 4,
                        "total_tokens": 14,
                    }
                )
            },
        }
    )
    payload = collector.event_payload()
    assert payload is not None
    assert payload["schema_version"] == 1
    assert len(payload["measurements"]) == 1
    assert payload["measurements"][0]["total_tokens"] == 14
    assert payload["measurements"][0]["model"] == "actual-model"


def test_collector_preserves_distinct_models_and_cache_details() -> None:
    collector = UsageCollector()
    for run_id, model, total in (("a", "main", 5), ("b", "subagent", 7)):
        collector.observe(
            {
                "event": "on_chat_model_end",
                "run_id": run_id,
                "metadata": {"ls_model_name": model},
                "data": {
                    "output": SimpleNamespace(
                        response_metadata={
                            "token_usage": {
                                "prompt_tokens": total - 2,
                                "completion_tokens": 2,
                                "total_tokens": total,
                                "cached_input_tokens": 1,
                            }
                        }
                    )
                },
            }
        )
    payload = collector.event_payload()
    assert payload is not None
    assert [item["model"] for item in payload["measurements"]] == ["main", "subagent"]
    assert payload["measurements"][0]["cache_read_tokens"] == 1


def test_collector_ignores_tools_and_missing_usage_metadata() -> None:
    collector = UsageCollector()
    collector.observe({"event": "on_tool_end", "run_id": "tool", "data": {}})
    collector.observe(
        {"event": "on_chat_model_end", "run_id": "model", "data": {"output": {}}}
    )
    assert collector.event_payload() is None
