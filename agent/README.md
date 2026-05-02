# MisakaX Agent Sidecar

Python Sidecar for MisakaX, providing LangGraph Agent orchestration and PowerMem memory engine.

## Quick Start

```bash
pip install -r requirements.txt
uvicorn app.main:app --host 127.0.0.1 --port 9527
```

## Health Check

```bash
curl http://127.0.0.1:9527/health
```
