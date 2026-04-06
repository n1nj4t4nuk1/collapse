FROM node:22-slim AS frontend-build

WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
ENV VITE_API_URL=""
RUN npm run build


FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

WORKDIR /app

COPY backend/pyproject.toml ./
COPY backend/app ./app
COPY backend/main.py ./

RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir .

COPY --from=frontend-build /frontend/dist ./static

EXPOSE 8000

CMD ["python", "main.py"]
