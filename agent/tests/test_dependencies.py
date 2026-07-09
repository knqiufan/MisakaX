"""Tests for checkpointer lifecycle helpers."""

from __future__ import annotations

import pytest

from app import dependencies


@pytest.fixture(autouse=True)
def _reset_checkpointer_state():
    dependencies._checkpointer = None
    dependencies._checkpointer_cm = None
    yield
    dependencies._checkpointer = None
    dependencies._checkpointer_cm = None


@pytest.mark.asyncio
async def test_setup_and_close_checkpointer(tmp_path, monkeypatch):
    monkeypatch.setenv("MISAKA_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("MISAKA_CHECKPOINTER_DB_PATH", str(tmp_path / "checkpoints.db"))
    from app.config import get_settings

    get_settings.cache_clear()

    await dependencies.setup_checkpointer()
    saver = dependencies.get_checkpointer()
    assert saver is not None
    assert getattr(saver, "conn", None) is not None

    # setup() enables WAL; verify via PRAGMA when connection is available.
    cursor = await saver.conn.execute("PRAGMA journal_mode;")
    row = await cursor.fetchone()
    assert row is not None
    assert str(row[0]).lower() == "wal"

    await dependencies.close_checkpointer()
    assert dependencies.get_checkpointer() is None
    get_settings.cache_clear()


@pytest.mark.asyncio
async def test_setup_checkpointer_import_error(monkeypatch):
    import builtins

    real_import = builtins.__import__

    def fake_import(name, *args, **kwargs):
        if name.startswith("langgraph.checkpoint.sqlite"):
            raise ImportError("missing")
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", fake_import)
    await dependencies.setup_checkpointer()
    assert dependencies.get_checkpointer() is None


@pytest.mark.asyncio
async def test_thread_isolation_with_checkpointer(tmp_path, monkeypatch):
    monkeypatch.setenv("MISAKA_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("MISAKA_CHECKPOINTER_DB_PATH", str(tmp_path / "checkpoints.db"))
    from app.config import get_settings

    get_settings.cache_clear()
    await dependencies.setup_checkpointer()
    saver = dependencies.get_checkpointer()
    assert saver is not None

    from langchain_core.messages import HumanMessage
    from langgraph.graph import END, START, MessagesState, StateGraph

    def echo(state: MessagesState):
        return {"messages": state["messages"]}

    graph = StateGraph(MessagesState)
    graph.add_node("echo", echo)
    graph.add_edge(START, "echo")
    graph.add_edge("echo", END)
    app = graph.compile(checkpointer=saver)

    cfg_a = {"configurable": {"thread_id": "thread-a"}}
    cfg_b = {"configurable": {"thread_id": "thread-b"}}
    await app.ainvoke({"messages": [HumanMessage(content="hello-a")]}, config=cfg_a)
    await app.ainvoke({"messages": [HumanMessage(content="hello-b")]}, config=cfg_b)

    state_a = await app.aget_state(cfg_a)
    state_b = await app.aget_state(cfg_b)
    assert state_a.values["messages"][0].content == "hello-a"
    assert state_b.values["messages"][0].content == "hello-b"

    await dependencies.close_checkpointer()

    # Re-open and ensure same thread restores.
    await dependencies.setup_checkpointer()
    saver2 = dependencies.get_checkpointer()
    app2 = graph.compile(checkpointer=saver2)
    restored = await app2.aget_state(cfg_a)
    assert restored.values["messages"][0].content == "hello-a"

    await dependencies.close_checkpointer()
    get_settings.cache_clear()
