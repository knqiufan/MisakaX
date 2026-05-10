"""FastAPI entry point for MisakaX Agent Sidecar."""

import time
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.config import settings
from app.routers.agent import router as agent_router
from app.routers.health import router as health_router
from app.routers.info import router as info_router


@asynccontextmanager
async def lifespan(application: FastAPI):
    """Manage startup and shutdown lifecycle."""
    application.state.startup_time = time.time()
    yield


app = FastAPI(
    title="MisakaX Agent",
    version="0.1.0",
    description="MisakaX Agent Sidecar - LangGraph + PowerMem",
    debug=settings.debug,
    lifespan=lifespan,
)

app.include_router(health_router)
app.include_router(info_router)
app.include_router(agent_router)
