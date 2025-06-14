# 📁 File Sharing API with Google OAuth (Rust + Actix Web)

This is a file-sharing backend written in Rust. It supports file upload, download, metadata, deletion, and Google OAuth 2.0 authentication.

---

## 🚀 Features

- 📥 Upload files to disk and store metadata in DB  
- 📤 Download files with correct headers  
- 🔍 Search files by name  
- 🗑️ Delete only own files  
- 🔐 Google OAuth 2.0 authentication  
- ✅ JWT-based protected routes  

---

## 📦 Tech Stack

- **Rust**
- **Actix Web**
- **Diesel**
- **PostgreSQL**
- **Google OAuth2**
- **JWT**

---

## ⚙️ Environment Configuration

Use the following `.env` structure (see `env.example`):

```env
DATABASE_URL=postgres://user:password@localhost/dbname
CLIENT_ID=your_google_client_id
CLIENT_SECRET=your_google_client_secret
REDIRECT_URI=http://localhost:8080/auth/google/callback
JWT_SECRET=super_secret_key
```

---

## 📚 API Endpoints

### 🔐 Authentication

#### `GET /auth/google`
Redirects to Google OAuth login.

#### `GET /auth/google/callback`
Handles OAuth callback, sets `auth_token` cookie.

#### `POST /auth/protected`
Protected route, requires `auth_token` cookie.

**Response:**
```json
{
  "message": "Hello, user_id: <UUID>"
}
```

---

### 📁 File Operations

#### `GET /api/files`
Returns list of all uploaded files.

**Example Response:**
```json
[
  {
    "id": 1,
    "name": "example.png",
    "mime_type": "image/png",
    "size": 42112,
    "created_at": "2025-06-13T12:00:00"
  }
]
```

#### `GET /api/files/search?q=report`
Search for files by name.

**Example Response:**
```json
[
  {
    "id": 2,
    "name": "report.pdf",
    "mime_type": "application/pdf",
    "size": 234567,
    "created_at": "2025-06-13T14:32:11"
  }
]
```

#### `POST /api/files`
Upload a file (multipart/form-data). Requires `auth_token`.

**Response:**
```json
"File uploaded successfully"
```

#### `GET /api/files/{id}`
Download a file by ID.

#### `GET /api/files/{id}/meta`
Returns file metadata.

**Response:**
```json
{
  "name": "file.txt",
  "mime_type": "text/plain",
  "size": 1234,
  "created_at": "2025-06-13 10:25:30"
}
```

#### `DELETE /api/files/{id}`
Deletes a file. Only the owner can delete.

**Response:**
```json
"File deleted successfully"
```

---

## 🧾 Example curl usage

### Google Auth
```bash
curl -v http://localhost:8080/auth/google
```

### Protected route
```bash
curl -H "Cookie: auth_token=your_jwt_here" \
     http://localhost:8080/auth/protected
```

### Upload file
```bash
curl -X POST http://localhost:8080/api/files \
     -H "Cookie: auth_token=your_jwt_here" \
     -F "file=@path/to/yourfile.png"
```

---

# 📁 API для обмена файлами с Google OAuth (Rust + Actix Web)

Это backend для обмена файлами на Rust. Поддерживает загрузку, скачивание, метаданные, удаление и аутентификацию через Google OAuth 2.0.

---

## 🚀 Возможности

- 📥 Загрузка файлов на диск и сохранение метаданных в БД  
- 📤 Скачивание файлов с корректными заголовками  
- 🔍 Поиск файлов по имени  
- 🗑️ Удаление только своих файлов  
- 🔐 Аутентификация через Google OAuth 2.0  
- ✅ Защищённые маршруты через JWT  

---

## 📦 Стек технологий

- **Rust**
- **Actix Web**
- **Diesel**
- **PostgreSQL**
- **Google OAuth2**
- **JWT**

---

## ⚙️ Конфигурация окружения

Создайте файл `.env` по примеру ниже (см. `env.example`):

```env
DATABASE_URL=postgres://user:password@localhost/dbname
CLIENT_ID=your_google_client_id
CLIENT_SECRET=your_google_client_secret
REDIRECT_URI=http://localhost:8080/auth/google/callback
JWT_SECRET=super_secret_key
```

---

## 📚 API Эндпоинты

### 🔐 Аутентификация

#### `GET /auth/google`
Перенаправляет на авторизацию Google.

#### `GET /auth/google/callback`
Обрабатывает callback и устанавливает cookie `auth_token`.

#### `POST /auth/protected`
Пример защищённого маршрута. Требуется cookie `auth_token`.

**Ответ:**
```json
{
  "message": "Hello, user_id: <UUID>"
}
```

---

### 📁 Работа с файлами

#### `GET /api/files`
Список всех загруженных файлов.

**Пример ответа:**
```json
[
  {
    "id": 1,
    "name": "example.png",
    "mime_type": "image/png",
    "size": 42112,
    "created_at": "2025-06-13T12:00:00"
  }
]
```

#### `GET /api/files/search?q=report`
Поиск файлов по имени.

**Пример ответа:**
```json
[
  {
    "id": 2,
    "name": "report.pdf",
    "mime_type": "application/pdf",
    "size": 234567,
    "created_at": "2025-06-13T14:32:11"
  }
]
```

#### `POST /api/files`
Загрузка файла (multipart/form-data). Требуется cookie `auth_token`.

**Ответ:**
```json
"File uploaded successfully"
```

#### `GET /api/files/{id}`
Скачивание файла по ID.

#### `GET /api/files/{id}/meta`
Метаданные файла.

**Ответ:**
```json
{
  "name": "file.txt",
  "mime_type": "text/plain",
  "size": 1234,
  "created_at": "2025-06-13 10:25:30"
}
```

#### `DELETE /api/files/{id}`
Удаление файла. Только владелец может удалить.

**Ответ:**
```json
"File deleted successfully"
```

---

## 🧾 Примеры curl-запросов

### Авторизация через Google
```bash
curl -v http://localhost:8080/auth/google
```

### Защищённый маршрут
```bash
curl -H "Cookie: auth_token=your_jwt_here" \
     http://localhost:8080/auth/protected
```

### Загрузка файла
```bash
curl -X POST http://localhost:8080/api/files \
     -H "Cookie: auth_token=your_jwt_here" \
     -F "file=@path/to/yourfile.png"
```
