"""PowerMem REST endpoints for search / list / delete."""

from __future__ import annotations

import logging
from typing import Any

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, Field

from app.memory import get_memory_engine

router = APIRouter(prefix="/memory", tags=["memory"])
logger = logging.getLogger(__name__)


class MemoryItem(BaseModel):
    id: str | None = None
    content: str = ""
    score: float | None = None
    created_at: str | None = None
    metadata: dict[str, Any] = Field(default_factory=dict)


class MemoryListResponse(BaseModel):
    items: list[MemoryItem]
    offset: int = 0
    limit: int = 20


def _require_engine() -> Any:
    engine = get_memory_engine()
    if engine is None:
        raise HTTPException(status_code=503, detail="Memory engine unavailable")
    return engine


def _normalize_item(item: Any) -> MemoryItem:
    if isinstance(item, dict):
        return MemoryItem(
            id=_as_optional_str(item.get("id") or item.get("memory_id")),
            content=str(item.get("content") or item.get("memory") or ""),
            score=_as_optional_float(item.get("score")),
            created_at=_as_optional_str(item.get("created_at")),
            metadata=item.get("metadata") if isinstance(item.get("metadata"), dict) else {},
        )

    return MemoryItem(
        id=_as_optional_str(getattr(item, "id", None) or getattr(item, "memory_id", None)),
        content=str(
            getattr(item, "content", None) or getattr(item, "memory", None) or ""
        ),
        score=_as_optional_float(getattr(item, "score", None)),
        created_at=_as_optional_str(getattr(item, "created_at", None)),
        metadata=getattr(item, "metadata", None)
        if isinstance(getattr(item, "metadata", None), dict)
        else {},
    )


def _as_optional_str(value: Any) -> str | None:
    if value is None:
        return None
    return str(value)


def _as_optional_float(value: Any) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def _call_search(engine: Any, query: str, limit: int) -> list[Any]:
    if hasattr(engine, "search"):
        result = engine.search(query, limit=limit)
    elif hasattr(engine, "query"):
        result = engine.query(query, limit=limit)
    else:
        raise HTTPException(status_code=503, detail="Memory engine has no search API")
    return list(result or [])


def _call_list(engine: Any, offset: int, limit: int) -> list[Any]:
    if hasattr(engine, "list"):
        result = engine.list(offset=offset, limit=limit)
    elif hasattr(engine, "get_all"):
        result = engine.get_all()
        result = list(result or [])[offset : offset + limit]
    else:
        raise HTTPException(status_code=503, detail="Memory engine has no list API")
    return list(result or [])


def _call_delete(engine: Any, memory_id: str) -> None:
    if hasattr(engine, "delete"):
        engine.delete(memory_id)
        return
    if hasattr(engine, "remove"):
        engine.remove(memory_id)
        return
    raise HTTPException(status_code=503, detail="Memory engine has no delete API")


@router.get("/search", response_model=MemoryListResponse)
async def memory_search(
    query: str = Query(..., min_length=1),
    limit: int = Query(5, ge=1, le=100),
) -> MemoryListResponse:
    engine = _require_engine()
    try:
        items = [_normalize_item(item) for item in _call_search(engine, query, limit)]
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("memory search failed")
        raise HTTPException(status_code=500, detail=str(exc)) from exc
    return MemoryListResponse(items=items, offset=0, limit=limit)


@router.get("/list", response_model=MemoryListResponse)
async def memory_list(
    offset: int = Query(0, ge=0),
    limit: int = Query(20, ge=1, le=100),
) -> MemoryListResponse:
    engine = _require_engine()
    try:
        items = [_normalize_item(item) for item in _call_list(engine, offset, limit)]
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("memory list failed")
        raise HTTPException(status_code=500, detail=str(exc)) from exc
    return MemoryListResponse(items=items, offset=offset, limit=limit)


@router.delete("/{memory_id}")
async def memory_delete(memory_id: str) -> dict[str, str]:
    engine = _require_engine()
    try:
        _call_delete(engine, memory_id)
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("memory delete failed")
        raise HTTPException(status_code=500, detail=str(exc)) from exc
    return {"status": "deleted", "id": memory_id}
