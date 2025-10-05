# Market - Prediction Market Platform Roadmap

## ✅ Completed

### Core Infrastructure
- [x] Database schema with SQLite
- [x] User management (signup/login)
- [x] Market creation and listing
- [x] Position tracking
- [x] Basic authentication (username/password)

### Session Management
- [x] Cookie-based session handling with `tower_sessions`
- [x] Track logged-in users across requests (`RequireAuth` & `OptionalAuth` extractors)
- [x] Working logout functionality
- [x] Session-based user tracking (no hardcoded user IDs)
- [x] Logout button in UI templates
- [x] User profile dropdown in navbar

### LMSR Implementation (Polymarket-style)
- [x] Logarithmic Market Scoring Rule (LMSR) pricing algorithm
- [x] Unlimited liquidity (shares minted/burned on demand)
- [x] Liquidity parameter `b` for market depth control
- [x] Migration from CPMM to LMSR
- [x] Update all trading handlers to use LMSR
- [x] Probability calculations based on outstanding shares
- [x] Comprehensive test suite for LMSR

### Trading
- [x] Buy shares functionality
- [x] Sell shares functionality
- [x] Position management
- [x] Profit/loss calculations
- [x] Market resolution
- [x] Real-time price preview before trade
- [x] Cost calculation API endpoint
- [x] Potential profit/loss display
- [x] Position tracking page with P&L

### Market Features
- [x] Oracle resolution system (designated resolver)
- [x] Oracle assignment on market creation
- [x] Market creator controls (can resolve if oracle not set)
- [x] Payout processing for resolved markets

### Analytics & Display
- [x] Price charts with Chart.js
- [x] Historical price data tracking
- [x] Price snapshot recording on each trade
- [x] API endpoint for price history
- [x] Position value tracking

### User Experience
- [x] Multi-theme system (light, dark, hacker, sepia, pastel)
- [x] Theme persistence with localStorage
- [x] User profiles with balance display
- [x] Current positions display on market detail page
- [x] Responsive navbar with user dropdown

## 🚧 High Priority (Core Functionality)

### Market Features
- [ ] Market categories/tags
- [ ] Market search and filtering
- [ ] Market history and activity feed
- [ ] Market validation before resolution
- [ ] Disputed resolution mechanism
- [ ] Market end date validation (auto-close trading)
- [ ] Market description rich text support

### Trading UX
- [ ] Slippage warnings (% price impact)
- [ ] Order limits and validation
- [ ] Trade history per user (activity log)
- [ ] Session expiration and refresh
- [ ] "Remember me" functionality

## 📊 Medium Priority (Enhancements)

### Analytics & Display
- [ ] Market statistics (volume, unique traders, etc.)
- [ ] Leaderboard (most profitable traders)
- [ ] Portfolio value tracking over time
- [ ] Daily/weekly P&L summaries
- [ ] Trading volume charts
- [ ] Market activity timeline

### User Experience
- [ ] Avatar/display name support
- [ ] Email notifications for market events
- [ ] Market comments/discussion
- [ ] Follow/favorite markets
- [ ] Mobile responsive improvements
- [ ] Accessibility enhancements

### Market Maker Improvements
- [ ] Dynamic liquidity parameter adjustment
- [ ] Market maker fees (spread)
- [ ] Liquidity provider rewards
- [ ] Multiple liquidity pools per market

## 🔧 Low Priority (Nice to Have)

### Advanced Features
- [ ] API endpoints for programmatic trading
- [ ] WebSocket support for real-time updates
- [ ] Market creation fees
- [ ] Referral system
- [ ] Multi-currency support
- [ ] Conditional markets (dependent on other markets)

### Technical Improvements
- [ ] Rate limiting
- [ ] Pagination for market lists
- [ ] Caching layer (Redis)
- [ ] Database connection pooling optimization
- [ ] Automated market maker parameter tuning
- [ ] Backup and disaster recovery

### Admin Tools
- [ ] Admin dashboard
- [ ] Market moderation tools
- [ ] User management (ban/suspend)
- [ ] System health monitoring
- [ ] Audit logs

## 🐛 Known Issues

### Database
- **Legacy fields**: `yes_pool` and `no_pool` columns still exist (kept for backward compatibility, not used)
- **No database migrations**: Schema changes require manual intervention

### UI/Templates
- **Error handling**: Failed states could be better displayed with proper error pages
- **Mobile responsiveness**: Not fully optimized for mobile devices
- **Form validation**: Client-side validation is minimal

## 🎯 Next Sprint Focus

1. **Market Discovery & Organization** (High Priority)
   - Add market categories/tags
   - Implement search functionality
   - Add filtering (active/resolved/by category)
   - Show market statistics (volume, traders)

2. **Trading Experience Enhancements**
   - Add slippage/price impact warnings
   - Trade history per user
   - Order validation improvements
   - Session timeout handling

3. **UI/UX Polish**
   - Better error pages and messages
   - Form validation improvements
   - Loading states for async operations
   - Mobile responsive refinements

## 📝 Technical Debt

- Remove or migrate away from legacy `yes_pool`/`no_pool` database columns
- Add proper error handling middleware (currently using basic Result/String errors)
- Implement comprehensive request validation layer
- Add integration tests for trading flows
- Add end-to-end tests for critical paths
- Document API endpoints (OpenAPI/Swagger)
- Add deployment documentation
- Database migration system (currently manual SQL changes)

## 🔮 Future Exploration

- Hybrid AMM + Order Book (like current Polymarket)
- Cross-chain support
- Decentralized resolution (oracle integration)
- AI-powered market suggestions
- Social features (groups, challenges)
- Tournament/competition mode
