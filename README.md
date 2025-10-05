# market

simple & extensible prediction markets built with rust

## features

- **lmsr pricing** - polymarket-style logarithmic market scoring rule with unlimited liquidity
- **instant trading** - buy and sell shares at algorithmically determined fair prices
- **oracle resolution** - designated resolvers ensure accurate market outcomes
- **price charts** - real-time probability tracking with historical data
- **multi-theme ui** - choose from light, dark, hacker, sepia, and pastel themes
- **session management** - secure cookie-based authentication

## tech stack

- **rust** - blazing fast & memory safe
- **axum** - modern web framework
- **sqlite** - embedded database via sqlx
- **askama** - type-safe html templates
- **chart.js** - interactive price charts
- **htmx** - dynamic ui without heavy javascript

## quick start

### using docker (recommended)

```bash
# pull and run the latest image
docker run -d \
  -p 3000:3000 \
  -v market-data:/app/data \
  ghcr.io/ludeed/market:latest

# open browser
open http://localhost:3000
```

### building from source

```bash
# clone the repo
git clone https://github.com/ludeed/market.git
cd market

# build and run with docker
docker build -t market .
docker run -p 3000:3000 -v market-data:/app/data market

# or run directly with cargo
cargo run
```

## how it works

### creating markets

anyone can create a binary prediction market with:
- a yes/no question
- an end date
- an optional oracle (resolver)

markets use lmsr (logarithmic market scoring rule) for pricing, which provides:
- unlimited liquidity (no liquidity pools to drain)
- fair pricing based on outstanding shares
- smooth price discovery

### trading

buy or sell shares at any time:
- see real-time cost preview before trading
- track your positions and p&l
- view historical price charts

### resolution

when a market ends:
- the designated oracle (or creator) resolves the outcome
- winning shares pay out $1 each
- losing shares are worthless
- profits are automatically credited

## project structure

```
market/
├── src/
│   ├── domain/          # core business logic (lmsr, positions, markets)
│   ├── repository/      # database access layer
│   ├── web/            # http handlers, routing, sessions
│   └── db/             # database utilities
├── templates/          # askama html templates
├── static/            # css and client-side assets
└── migrations/        # sql schema
```

## configuration

the app uses environment variables:

```bash
DATABASE_URL=sqlite:market.db  # database path
HOST=0.0.0.0                   # bind address
PORT=3000                      # port number
```

## development

```bash
# run with auto-reload
cargo watch -x run

# run tests
cargo test

# format code
cargo fmt

# lint
cargo clippy
```

## roadmap

see [ROADMAP.md](ROADMAP.md) for planned features and known issues

## credits

built by [ludeed](https://github.com/ludeed) & [claudius](https://claude.ai)

## license

MIT
