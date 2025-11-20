# TensorZero Setup for RAK

This directory contains the TensorZero gateway configuration for RAK integration.

## 🚀 Quick Start

### 1. Setup Environment Variables

```bash
# Copy the example environment file
cp env.example .env

# Edit .env and add your API keys
vim .env
```

**Required Configuration:**
- `OPENAI_API_KEY` - Required for TensorZero gateway
- `GEMINI_API_KEY` - For RAK/Gemini integration (optional)
- `ANTHROPIC_API_KEY` - For Anthropic models (optional)
- `CLICKHOUSE_USER` - Database username (default: chusertoxi)
- `CLICKHOUSE_PASSWORD` - Database password (default: chpasswordtoxi$43)

### 2. Start Services

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Check status
docker-compose ps
```

### 3. Access Services

Once running, you can access:

- **TensorZero Gateway**: http://localhost:3000
- **TensorZero UI**: http://localhost:4000
- **ClickHouse**: http://localhost:8123

### 4. Stop Services

```bash
# Stop all services (data persists in ./data/)
docker-compose down

# Stop and remove containers
docker-compose down

# To completely remove data (careful!)
docker-compose down && rm -rf data/clickhouse/*
```

## 📁 Directory Structure

```
t0/
├── docker-compose.yml   # Service configuration
├── .env                 # Your API keys (not in git)
├── env.example          # Template for .env
├── .gitignore           # Ignores .env and data/
├── data/                # Database storage (persisted locally, not in git)
│   ├── .gitkeep         # Keeps directory structure in git
│   └── clickhouse/      # ClickHouse data files
├── config/              # TensorZero configuration
│   └── tensorzero.toml
└── README.md            # This file
```

## 🔧 Configuration

### Environment Variables

The `.env` file is automatically loaded by docker-compose. It should contain:

```env
# API Keys
OPENAI_API_KEY=sk-...
GEMINI_API_KEY=...
ANTHROPIC_API_KEY=...

# Database Credentials
CLICKHOUSE_USER=chusertoxi
CLICKHOUSE_PASSWORD=chpasswordtoxi$43
```

**Important**: Use a strong password for production! The default password is only for development.

### Services

**ClickHouse** (Port 8123)
- Database for storing TensorZero data
- Credentials: Set in `.env` file (defaults: `chusertoxi` / `chpasswordtoxi$43`)
- Data stored in: `./data/clickhouse/` (persisted locally)
- ⚠️ Change default password for production!

**Gateway** (Port 3000)
- TensorZero gateway service
- Exposes HTTP API
- Required for OpenAI client compatibility

**UI** (Port 4000)
- TensorZero web interface
- Monitor and debug inference requests

## 💾 Data Persistence

### Local Storage

Database data is stored in `./data/clickhouse/` on your local machine:

✅ **Benefits**:
- Data survives Docker restarts
- Easy to backup (just copy the folder)
- Easy to restore (copy back the folder)
- Can inspect/debug data files directly
- Portable across Docker installations

### Backup

```bash
# Backup database
tar -czf clickhouse-backup-$(date +%Y%m%d).tar.gz data/clickhouse/

# Restore from backup
docker-compose down
rm -rf data/clickhouse/*
tar -xzf clickhouse-backup-YYYYMMDD.tar.gz
docker-compose up -d
```

### Reset Database

```bash
# Stop services
docker-compose down

# Remove all data
rm -rf data/clickhouse/*

# Start fresh
docker-compose up -d

docker-compose run --rm gateway --run-clickhouse-migrations
docker-compose run --rm gateway --run-postgres-migrations
docker compose run --rm gateway --create-api-key
```

## 🛡️ Security Notes

⚠️ **Important:**
- ✅ `.env` is in `.gitignore` - Your API keys are safe
- ✅ `data/` is in `.gitignore` - Database not committed
- ✅ Use `env.example` as a template (no real keys)
- ❌ Never commit `.env` to git
- ❌ Never commit `data/` folder to git
- ❌ This setup is for **development only**
- ❌ For production, see: https://www.tensorzero.com/docs/deployment/tensorzero-gateway

### Production Checklist

Before deploying to production:

- [ ] Change default ClickHouse password
- [ ] Use strong passwords (16+ characters)
- [ ] Consider encrypted volumes
- [ ] Set up regular backups
- [ ] Use proper firewall rules
- [ ] Enable SSL/TLS for connections
- [ ] Review TensorZero production guide

## 🐛 Troubleshooting

### Services won't start

```bash
# Check logs
docker-compose logs

# Restart specific service
docker-compose restart gateway

# Rebuild images
docker-compose up --build
```

### API Key errors

```bash
# Verify .env file exists
ls -la .env

# Check if docker-compose loads it
docker-compose config
```

### Port conflicts

If ports 3000, 4000, or 8123 are already in use, edit `docker-compose.yml`:

```yaml
ports:
  - "3001:3000"  # Change 3000 to 3001
```

### ClickHouse not healthy

```bash
# Check ClickHouse logs
docker-compose logs clickhouse

# Manually test connection (use your credentials from .env)
curl http://chusertoxi:chpasswordtoxi$43@localhost:8123/ping

# Test with environment variables
source .env
curl http://${CLICKHOUSE_USER}:${CLICKHOUSE_PASSWORD}@localhost:8123/ping
```

### Data permission errors

```bash
# Fix permissions on data folder
sudo chown -R $(id -u):$(id -g) data/

# Or run with appropriate user
docker-compose down
rm -rf data/clickhouse/*
docker-compose up -d
```

### Database corruption

```bash
# Stop services
docker-compose down

# Backup corrupted data (optional)
mv data/clickhouse data/clickhouse.backup

# Create fresh data directory
mkdir -p data/clickhouse

# Start with clean database
docker-compose up -d
```

## 📚 Resources

- [TensorZero Documentation](https://www.tensorzero.com/docs)
- [TensorZero Gateway](https://www.tensorzero.com/docs/deployment/tensorzero-gateway)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [ClickHouse Documentation](https://clickhouse.com/docs)

## 🔗 Integration with RAK

To use TensorZero with RAK:

1. Start TensorZero services: `docker-compose up -d`
2. Use the gateway URL in your RAK config: `http://localhost:3000`
3. Configure RAK to use TensorZero as an LLM provider

See RAK documentation for integration details.

## 🎯 Common Tasks

### View Database Size

```bash
# Check data directory size
du -sh data/clickhouse/

# Detailed breakdown
du -h data/clickhouse/ | sort -rh | head -20
```

### Clean Old Logs

```bash
# View logs
docker-compose logs --tail=100

# Clear logs
docker-compose down
rm -rf data/clickhouse/store/*/log*.log
docker-compose up -d
```

### Export Data

```bash
# Export as SQL dump (requires clickhouse-client)
docker-compose exec clickhouse clickhouse-client --user=${CLICKHOUSE_USER} --password=${CLICKHOUSE_PASSWORD} --query="SELECT * FROM tensorzero.inference_table FORMAT CSV" > export.csv
```

### Monitor Services

```bash
# Watch logs in real-time
docker-compose logs -f

# Check resource usage
docker stats

# View service health
docker-compose ps
```
