# Abuse-Helper

![Build Status](https://github.com/habibayoub/Abuse-Helper/workflows/Rust%20CI/badge.svg)

Abuse-Helper is a comprehensive security incident management system designed to automate the handling of abuse reports, threat analysis, and incident response workflows.

## CI Status

| Job       | Status                                                                                                                    |
|-----------|---------------------------------------------------------------------------------------------------------------------------|
| Build     | ![Build](https://github.com/habibayoub/Abuse-Helper/actions/workflows/rust-ci.yml/badge.svg?event=push&job=build)   |
| Test      | ![Test](https://github.com/habibayoub/Abuse-Helper/actions/workflows/rust-ci.yml/badge.svg?event=push&job=test)     |
| Clippy    | ![Clippy](https://github.com/habibayoub/Abuse-Helper/actions/workflows/rust-ci.yml/badge.svg?event=push&job=clippy) |


## Features

- **Email Integration**
  - IMAP/SMTP support for email fetching and sending
  - Automated email processing and analysis
  - Email-to-ticket conversion

- **Threat Analysis**
  - LLM-powered content analysis
  - Threat indicator extraction
  - Confidence scoring
  - Automated categorization

- **Ticket Management**
  - Customizable ticket workflows
  - Email association
  - Status tracking
  - Threat categorization

- **Search & Analytics**
  - Full-text search via Elasticsearch
  - Advanced filtering options
  - Threat pattern analysis
  - Historical data tracking

- **Security**
  - Keycloak authentication
  - JWT authentication
  - Role-based access control
  - Activity logging
  - Audit trails

## System Requirements

### Minimum
- RAM: 8GB
- CPU: 4 cores
- Storage: 20GB
- Docker Engine
- Docker Compose

## Quick Start

1. Ensure Docker and Docker Compose are installed
2. Clone the repository
3. In the project root, run the Docker Compose file to pull dependencies and start the containers:
```shell
docker compose up --build
```
3. Visit http://localhost:8080 and create an "admin" (admin@example.com:admin123) user in the Abuse-Helper realm with the [admin, user] roles
4. Visit http://localhost:3000 to get to the login page
5. Log into the frontend using the above credentials

## Docker

### Containers

| Service | Image | Description | Ports |
|---------|-------|-------------|-------|
| frontend | node:18 | React application | 3000 |
| backend | rust:1.70 | Actix Web API | 8000 |
| postgres | postgres:15 | Main database | 5432 |
| elasticsearch | elasticsearch:8.7.1 | Search engine | 9200 |
| keycloak | keycloak:21.1 | Authentication | 8080 |
| keycloak-db | postgres:15 | Keycloak database | 5432 |
| mailserver | greenmail:2.0.0 | Email testing | 3025, 3993 |
| llm | ollama:latest | AI processing | 11434 |

### Volumes

| Volume | Service | Purpose |
|--------|---------|---------|
| postgres-data | postgres | Database persistence |
| elasticsearch-data | elasticsearch | Search index storage |
| keycloak-data | keycloak | Auth server data |
| llm-data | llm | Model storage |

### Networks

| Network | Purpose | Connected Services |
|---------|---------|-------------------|
| frontend-network | Frontend isolation | frontend |
| backend-network | Backend services | All except frontend |


## API

Example Login Request
```http
POST /api/auth/login
Content-Type: application/json
{
"email": "user@example.com",
"password": "your_password"
}
```
Example Login Response
```json
{
"token": "eyJhbGciOiJIUzI1NiIs...",
"expires_in": 3600
}
```
Example Authenticated Request
```http
GET /api/email/list
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```
