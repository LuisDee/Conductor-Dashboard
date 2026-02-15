# Technology Stack

## Core Technologies
- **Backend Language:** Python 3.11+
- **Backend Framework:** FastAPI (Async REST API)
- **Frontend Framework:** React (Vite, TypeScript)
- **Styling Engine:** Tailwind CSS
    - *Note:* UI/UX implementation is flexible and open to modernization if it improves user experience.
- **Database:** PostgreSQL
    - **ORM:** SQLAlchemy (Async)
    - **Driver:** asyncpg
    - **Migrations:** Alembic

## Integrations & Services
- **Slack Integration:** Slack Bolt for Python (Socket Mode)
- **AI/LLM:** Google Generative AI (`google-generativeai`)
- **Task Scheduling:** APScheduler

## Infrastructure & Tooling
- **Containerization:** Docker & Docker Compose
- **Reverse Proxy:** Nginx
- **Testing:** Pytest (Backend), Playwright (Frontend E2E)
- **Linting/Formatting:** Ruff (Python), ESLint/Prettier (TypeScript)
