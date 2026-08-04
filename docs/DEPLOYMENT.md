# Deployment Guide

## Prerequisites

```bash
# Install Stellar CLI
curl -fsSL https://github.com/stellar/stellar-cli/raw/main/install.sh | sh

# Generate a testnet account
stellar keys generate --global deployer --network testnet
stellar keys address deployer

# Fund with friendbot
curl "https://friendbot.stellar.org?addr=$(stellar keys address deployer)"
```

## Deploy to Testnet

```bash
cp .env.example .env
# Set STELLAR_ACCOUNT=deployer in .env

bash scripts/deploy.sh
```

## Verify Deployment

```bash
# Query stream count
stellar contract invoke \
  --id $STREAM_CONTRACT_ID \
  --network testnet \
  -- claimable_amount --stream_id 0
```
