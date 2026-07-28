"""Tests for the Sidecar Uvicorn entry-point configuration."""

import logging

from app.config import Settings
from run import (
    SIDECAR_HTTP_KEEP_ALIVE_SECONDS,
    SuccessfulHealthCheckAccessFilter,
    build_uvicorn_config,
)


def _access_record(method: str, path: str, status_code: int) -> logging.LogRecord:
    return logging.LogRecord(
        "uvicorn.access",
        logging.INFO,
        __file__,
        1,
        '%s - "%s %s HTTP/%s" %d',
        ("127.0.0.1:12345", method, path, "1.1", status_code),
        None,
    )


def test_successful_health_access_logs_are_filtered():
    log_filter = SuccessfulHealthCheckAccessFilter()

    assert not log_filter.filter(_access_record("GET", "/health", 200))
    assert not log_filter.filter(_access_record("HEAD", "/health?verbose=1", 204))
    assert log_filter.filter(_access_record("GET", "/health", 500))
    assert log_filter.filter(_access_record("GET", "/agent/chat", 200))


def test_uvicorn_config_keeps_local_connections_alive():
    config = build_uvicorn_config(Settings(host="127.0.0.1", port=9527))

    assert config.timeout_keep_alive == SIDECAR_HTTP_KEEP_ALIVE_SECONDS
    assert config.access_log is True
