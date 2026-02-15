# PA Dealing AI Agent - Implementation Plan

## Overview
Production-grade multi-agent system for automating Personal Account Dealing compliance workflows.

## Architecture Decisions

### Database Strategy
- **ORM**: SQLAlchemy 2.0 with async support
- **DB Agnostic**: Same models work with Oracle and PostgreSQL
- **Testing**: Local PostgreSQL via Docker
- **Production**: Oracle (current) → PostgreSQL (future)

### Agent Framework
- **Framework**: Google ADK (Agent Development Kit)
- **Model**: Gemini 2.0 Flash (via Vertex AI)
- **Pattern**: All async, consistent throughout

### Audit Logging
- **Flexible Backend**: Database OR stdout (configurable)
- **Structured**: JSON format for parseability
- **Compliant**: 7-year retention capability (FR7)

---

## Implementation Phases

### Phase 1: Project Foundation
**Goal**: Set up project structure, database layer, and local dev environment

#### 1.1 Project Structure
- [ ] Create directory structure
- [ ] Set up pyproject.toml with dependencies
- [ ] Create .env.example and config management
- [ ] Set up logging infrastructure

#### 1.2 Database Layer
- [ ] SQLAlchemy models (DB-agnostic)
- [ ] Async connection management
- [ ] Alembic migrations setup
- [ ] Docker Compose for local PostgreSQL

#### 1.3 Test Data
- [ ] Create seed data scripts
- [ ] Fabricate realistic test employees
- [ ] Fabricate PAD request history
- [ ] Create restricted list test data

### Phase 2: Database Agent
**Goal**: Implement all database tools with full async support

#### 2.1 Core Tools
- [ ] get_employee_by_email
- [ ] get_employee_by_id
- [ ] get_manager_chain

#### 2.2 PAD Request Tools
- [ ] get_pad_requests
- [ ] submit_pad_request
- [ ] update_pad_status (manager)
- [ ] update_pad_status (compliance)

#### 2.3 Compliance Check Tools
- [ ] check_restricted_list
- [ ] check_mako_positions
- [ ] check_holding_period
- [ ] check_recent_activity

#### 2.4 Testing
- [ ] Unit tests for all tools
- [ ] Integration tests with PostgreSQL
- [ ] Edge case coverage

### Phase 3: Audit Logging
**Goal**: Implement flexible audit trail system

#### 3.1 Audit Infrastructure
- [ ] Audit log table schema
- [ ] Audit log SQLAlchemy model
- [ ] Pluggable logging backends (DB, stdout, future: cloud)

#### 3.2 Integration
- [ ] Decorator for automatic tool auditing
- [ ] Context propagation (request_id, user, session)
- [ ] Structured log format

### Phase 4: Slack Agent
**Goal**: Implement Slack interaction layer

#### 4.1 Slack Infrastructure
- [ ] Slack Bolt async setup
- [ ] Event handlers (messages, button clicks)
- [ ] Signature verification middleware

#### 4.2 Conversation Tools
- [ ] send_dm
- [ ] post_to_channel
- [ ] update_message
- [ ] get_user_info

#### 4.3 Approval Workflows
- [ ] Block Kit message builders
- [ ] Manager approval flow
- [ ] Compliance approval flow
- [ ] SMF16 escalation flow

#### 4.4 Testing
- [ ] Mock Slack client for unit tests
- [ ] Integration tests with Slack sandbox

### Phase 5: Orchestrator Agent
**Goal**: Implement decision-making and coordination

#### 5.1 Risk Classification
- [ ] Policy rule engine
- [ ] LLM-based classification
- [ ] Explanation generation

#### 5.2 Workflow Orchestration
- [ ] Request intake flow
- [ ] Approval routing logic
- [ ] Status update handling

#### 5.3 Testing
- [ ] Test cases for each risk level
- [ ] Policy rule unit tests
- [ ] End-to-end workflow tests

### Phase 6: Integration & Deployment
**Goal**: Connect all components and prepare for deployment

#### 6.1 Agent Integration
- [ ] Wire up all agents
- [ ] End-to-end flow testing
- [ ] Error handling and recovery

#### 6.2 Deployment Preparation
- [ ] Dockerfile
- [ ] Kubernetes manifests (optional)
- [ ] Environment configuration guide

---

## File Structure

```
pa_dealing/
├── .env.example
├── .env                          # Local development (gitignored)
├── pyproject.toml
├── alembic.ini
├── docker-compose.yml            # Local PostgreSQL
├── Dockerfile
│
├── alembic/
│   └── versions/                 # Database migrations
│
├── src/
│   └── pa_dealing/
│       ├── __init__.py
│       ├── main.py               # Entry point
│       │
│       ├── config/
│       │   ├── __init__.py
│       │   └── settings.py       # Pydantic Settings
│       │
│       ├── db/
│       │   ├── __init__.py
│       │   ├── engine.py         # Async engine setup
│       │   ├── models.py         # SQLAlchemy models
│       │   └── session.py        # Session management
│       │
│       ├── audit/
│       │   ├── __init__.py
│       │   ├── logger.py         # Audit logger interface
│       │   ├── backends/
│       │   │   ├── __init__.py
│       │   │   ├── database.py   # DB backend
│       │   │   └── stdout.py     # Stdout backend
│       │   └── models.py         # Audit log model
│       │
│       ├── agents/
│       │   ├── __init__.py
│       │   ├── database/
│       │   │   ├── __init__.py
│       │   │   ├── agent.py
│       │   │   ├── tools.py
│       │   │   └── schemas.py
│       │   │
│       │   ├── slack/
│       │   │   ├── __init__.py
│       │   │   ├── agent.py
│       │   │   ├── tools.py
│       │   │   ├── blocks.py
│       │   │   └── handlers.py
│       │   │
│       │   └── orchestrator/
│       │       ├── __init__.py
│       │       ├── agent.py
│       │       ├── prompts.py
│       │       └── routing.py
│       │
│       └── models/
│           ├── __init__.py
│           ├── employee.py
│           ├── pad_request.py
│           └── risk_assessment.py
│
├── tests/
│   ├── __init__.py
│   ├── conftest.py               # Pytest fixtures
│   ├── test_db_agent.py
│   ├── test_slack_agent.py
│   ├── test_orchestrator.py
│   └── test_integration.py
│
└── scripts/
    ├── seed_data.py              # Generate test data
    └── run_local.py              # Local development runner
```

---

## Dependencies

```toml
[project]
name = "pa-dealing"
version = "0.1.0"
requires-python = ">=3.11"

dependencies = [
    # Database
    "sqlalchemy[asyncio]>=2.0",
    "asyncpg>=0.29",              # PostgreSQL async driver
    "alembic>=1.13",

    # Google AI
    "google-generativeai>=0.8",
    "google-cloud-aiplatform>=1.71",

    # Slack
    "slack-bolt>=1.18",
    "aiohttp>=3.9",

    # Core
    "pydantic>=2.0",
    "pydantic-settings>=2.0",
    "python-dotenv>=1.0",

    # Utilities
    "structlog>=24.0",
    "tenacity>=8.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0",
    "pytest-asyncio>=0.23",
    "pytest-cov>=4.0",
    "httpx>=0.27",                # For testing HTTP
    "faker>=24.0",                # For generating test data
]
oracle = [
    "oracledb>=2.0",              # Oracle async driver
]
```

---

## Execution Order

1. **Phase 1.1-1.2**: Foundation (this session)
2. **Phase 1.3**: Test data generation
3. **Phase 2**: Database Agent (use subagent for testing)
4. **Phase 3**: Audit logging
5. **Phase 4**: Slack Agent (after you provide credentials)
6. **Phase 5**: Orchestrator Agent
7. **Phase 6**: Integration

Each phase will be committed as a working increment.
