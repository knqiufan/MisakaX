"""FastAPI entry point for MisakaX Agent Sidecar."""

import time
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.config import bridge_provider_api_keys, get_settings
from app.dependencies import close_checkpointer, setup_checkpointer
from app.routers.agent import router as agent_router
from app.routers.health import cache_health_capabilities, router as health_router
from app.routers.info import router as info_router
from app.routers.memory import router as memory_router


@asynccontextmanager
async def lifespan(application: FastAPI):
    """Manage startup and shutdown lifecycle."""
    application.state.startup_time = time.time()
    cache_health_capabilities(application)
    bridge_provider_api_keys()
    await setup_checkpointer()
    try:
        yield
    finally:
        await close_checkpointer()


app = FastAPI(
    title="MisakaX Agent",
    version="0.1.0",
    description="MisakaX Agent Sidecar - LangGraph + PowerMem",
    debug=get_settings().debug,
    lifespan=lifespan,
)

app.include_router(health_router)
app.include_router(info_router)
app.include_router(agent_router)
app.include_router(memory_router)
