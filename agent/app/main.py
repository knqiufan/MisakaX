"""FastAPI entry point for MisakaX Agent Sidecar."""

from fastapi import FastAPI

from app.config import settings
from app.routers.health import router as health_router

app = FastAPI(
    title="MisakaX Agent",
    version="0.1.0",
    description="MisakaX Agent Sidecar - LangGraph + PowerMem",
)

app.include_router(health_router)


@app.on_event("startup")
async def startup():
    """Initialize resources on startup."""
    pass


@app.on_event("shutdown")
async def shutdown():
    """Cleanup resources on shutdown."""
    pass
