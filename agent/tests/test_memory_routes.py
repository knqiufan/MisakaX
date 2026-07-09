"""Tests for PowerMem REST routes."""

from __future__ import annotations

from unittest.mock import MagicMock, patch

from fastapi.testclient import TestClient

from app.main import app


def test_memory_search_503_when_unavailable():
    with patch("app.routers.memory.get_memory_engine", return_value=None):
        client = TestClient(app)
        resp = client.get("/memory/search", params={"query": "prefs"})
        assert resp.status_code == 503


def test_memory_search_success():
    engine = MagicMock()
    engine.search.return_value = [
        {"id": "m1", "content": "likes dark mode", "score": 0.9},
    ]
    with patch("app.routers.memory.get_memory_engine", return_value=engine):
        client = TestClient(app)
        resp = client.get("/memory/search", params={"query": "theme", "limit": 3})
        assert resp.status_code == 200
        body = resp.json()
        assert body["limit"] == 3
        assert body["items"][0]["id"] == "m1"
        assert body["items"][0]["content"] == "likes dark mode"
        engine.search.assert_called_once_with("theme", limit=3)


def test_memory_list_pagination():
    engine = MagicMock()
    engine.list.return_value = [{"id": "m2", "content": "item"}]
    with patch("app.routers.memory.get_memory_engine", return_value=engine):
        client = TestClient(app)
        resp = client.get("/memory/list", params={"offset": 5, "limit": 10})
        assert resp.status_code == 200
        body = resp.json()
        assert body["offset"] == 5
        assert body["limit"] == 10
        engine.list.assert_called_once_with(offset=5, limit=10)


def test_memory_delete_success():
    engine = MagicMock()
    with patch("app.routers.memory.get_memory_engine", return_value=engine):
        client = TestClient(app)
        resp = client.delete("/memory/m1")
        assert resp.status_code == 200
        assert resp.json()["status"] == "deleted"
        engine.delete.assert_called_once_with("m1")
