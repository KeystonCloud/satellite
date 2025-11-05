# KeystonCloud Satellite

## Development Setup
To set up a development environment for KeystonCloud, we use this simple files structure:
```
keystone-cloud/
 ├── node/
 ├── satellite/
 │    ├── Dockerfile.dev
 │    ├── start.sh
 │    ├── ...
 ├── webapp/
 ├── docker-compose.yml
```

### Define Compose file
If you want to use docker compose for development, you can add into ``services`` part all needed services for satellite. This is a simple example of configuration:
```yaml
  postgres:
    image: postgres:18
    restart: unless-stopped
    environment:
      POSTGRES_USER: keyston
      POSTGRES_PASSWORD: keystonpassword
      POSTGRES_DB: keyston_db
    volumes:
      - postgres-data:/var/lib/postgresql

  adminer:
    image: adminer
    restart: unless-stopped
    ports:
      - 8080:8080

  satellite:
    build:
      context: ./satellite
      dockerfile: Dockerfile.dev
    restart: unless-stopped
    environment:
      DATABASE_URL: postgres://keyston:keystonpassword@postgres:5432/keyston_db # For sqlx-cli
    volumes:
      - ./satellite:/app
    ports:
      - 8000:8000
    depends_on:
      - postgres
      - redis
    deploy:
      replicas: 1
```

Add new volume storage for postgres data in ``volumes`` part of your docker compose file:
```yaml
  postgres-data:
```

This stack will create a postgres database, an adminer service and the satellite service. The satellite service will be built using the `Dockerfile.dev` file located in the `satellite` folder and use starting script `start.sh`.
This starting script will run the application by using `cargo watch` to automatically reload the application when code changes are detected.


## Database cheatsheet
### Migrations
### Create new Migration
To create a new migration, you can use the following commands:
```bash
docker compose exec satellite sqlx migrate add <migration_name>
```

### Run Migrations
To run migrations, you can use the following commands:
```bash
docker compose exec satellite sqlx migrate run
```

### Revert Migration
To revert migration, you can use the following commands:
```bash
docker compose exec satellite sqlx migrate revert
```
