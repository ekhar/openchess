#!/bin/bash
set -e # Exit on error

# Configuration for the local Docker PostgreSQL instance
DB_NAME="chess_database"
DB_USER="admin"
DB_PASSWORD="9789"
DB_PORT="5432"
CONTAINER_NAME="chess-db-local"
SEED_FILE="scripts/seed.sql"
NETWORK_NAME="chess-network"

# Extract just the filename for use in the container
SEED_FILENAME=$(basename "$SEED_FILE")

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Setting up local chess database in Docker...${NC}"

# Check if Docker is available
if ! command -v docker &>/dev/null; then
  echo "Error: Docker is not installed or not in PATH"
  exit 1
fi

# Create Docker network if it doesn't exist
if ! docker network inspect $NETWORK_NAME &>/dev/null; then
  echo "Creating Docker network: $NETWORK_NAME"
  docker network create $NETWORK_NAME
fi

# Check if container already exists
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo "Container $CONTAINER_NAME already exists"

  # Check if container is running
  if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Starting existing container: $CONTAINER_NAME"
    docker start $CONTAINER_NAME
  fi
else
  echo "Creating and starting PostgreSQL container: $CONTAINER_NAME"
  docker run -d \
    --name $CONTAINER_NAME \
    --network $NETWORK_NAME \
    -e POSTGRES_USER=$DB_USER \
    -e POSTGRES_PASSWORD=$DB_PASSWORD \
    -e POSTGRES_DB=$DB_NAME \
    -p $DB_PORT:5432 \
    -v chess-db-data:/var/lib/postgresql/data \
    postgres:16

  # Wait for PostgreSQL to start
  echo "Waiting for PostgreSQL to start..."
  sleep 5
  MAX_RETRIES=30
  RETRIES=0

  until docker exec $CONTAINER_NAME pg_isready -U $DB_USER -h localhost || [ $RETRIES -eq $MAX_RETRIES ]; do
    echo "Waiting for PostgreSQL to become ready..."
    sleep 2
    RETRIES=$((RETRIES + 1))
  done

  if [ $RETRIES -eq $MAX_RETRIES ]; then
    echo "Error: PostgreSQL failed to start after $MAX_RETRIES retries"
    exit 1
  fi
fi

echo "PostgreSQL is running."

# Check if the database exists - connect to postgres database (always exists)
if docker exec $CONTAINER_NAME psql -U $DB_USER -d postgres -lqt | cut -d \| -f 1 | grep -qw $DB_NAME; then
  echo "Database $DB_NAME already exists."

  # Ask if we should drop and recreate
  read -p "Do you want to drop and recreate the database? (y/n) " -n 1 -r
  echo

  if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Dropping database $DB_NAME..."
    # Connect to postgres database to drop other databases
    docker exec -i $CONTAINER_NAME psql -U $DB_USER -d postgres -c "DROP DATABASE $DB_NAME;"
    echo "Creating database $DB_NAME..."
    docker exec -i $CONTAINER_NAME psql -U $DB_USER -d postgres -c "CREATE DATABASE $DB_NAME WITH ENCODING 'UTF8' LC_COLLATE='C' LC_CTYPE='C' TEMPLATE=template0;"
  else
    echo "Keeping existing database. Exiting."
    exit 0
  fi
else
  echo "Creating database $DB_NAME..."
  # Connect to postgres database to create other databases - with encoding settings
  docker exec -i $CONTAINER_NAME psql -U $DB_USER -d postgres -c "CREATE DATABASE $DB_NAME WITH ENCODING 'UTF8' LC_COLLATE='C' LC_CTYPE='C' TEMPLATE=template0;"
fi

# Copy the seed file to the container
echo "Copying seed file to container..."
docker cp "$SEED_FILE" "$CONTAINER_NAME:/tmp/$SEED_FILENAME"

# Apply the seed file with binary mode enabled
echo "Applying seed file to database..."
docker exec -i $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME -v ON_ERROR_STOP=0 -f "/tmp/$SEED_FILENAME"

# Verify that the seed was applied correctly
echo "Verifying seed data..."
docker exec -i $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME -c "
SELECT 'Games' as table_name, COUNT(*) as record_count FROM games UNION
SELECT 'Positions', COUNT(*) FROM positions UNION
SELECT 'Unique Positions', COUNT(*) FROM unique_positions UNION
SELECT 'Game Positions', COUNT(*) FROM game_positions
ORDER BY table_name;"

echo -e "\n${GREEN}Database setup complete!${NC}"
echo -e "Connection details:"
echo -e "  Host:     localhost or $CONTAINER_NAME (within Docker network)"
echo -e "  Port:     $DB_PORT"
echo -e "  Database: $DB_NAME"
echo -e "  User:     $DB_USER"
echo -e "  Password: $DB_PASSWORD"

# Create a .env file for local development
ENV_FILE=".env.local"
echo "Creating $ENV_FILE with database configuration..."
cat >$ENV_FILE <<EOF
# Chess Database Configuration
DB_HOST=localhost
DB_PORT=$DB_PORT
DB_NAME=$DB_NAME
DB_USER=$DB_USER
DB_PASSWORD=$DB_PASSWORD

# Docker Configuration
CHESS_DB_CONTAINER=$CONTAINER_NAME
CHESS_NETWORK=$NETWORK_NAME
EOF

echo -e "${GREEN}Created $ENV_FILE with connection details${NC}"
