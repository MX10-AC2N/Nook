# SOUL - Spirit of Nook Development

## Vision
Nook is not just another chat application. It's a privacy-first, feature-rich communication platform built with passion and precision. Our vision is to create a tool that combines the best of modern messaging with the reliability and security users deserve.

## Core Values

### 1. Quality Over Speed
We prioritize stable, well-tested code over rapid but fragile development. Every commit should leave the codebase better than we found it.

### 2. User Privacy & Security
Privacy is not an afterthought; it's the foundation. We implement end-to-end encryption, minimal data retention, and transparent privacy practices.

### 3. Developer Experience
We maintain a codebase that is clean, well-documented, and enjoyable to work with. New contributors should be able to understand and contribute quickly.

### 4. Performance Matters
Every millisecond counts. We optimize for responsiveness, low latency, and efficient resource usage across all platforms.

## Development Principles

### Code Quality

- **No broken windows**: Fix issues when you see them, don't postpone
- **Boy Scout Rule**: Leave the code cleaner than you found it
- **Single Responsibility**: Each component should have one clear purpose
- **Explicit over implicit**: Be clear about what the code does
- **You build it, you run it**: Developers are responsible for operating their code in production
- **Logs are for debugging, not for user communication**: Use proper error handling and user-friendly messages
- **Fail fast and loudly**: Make errors obvious and actionable
- **Consistency over cleverness**: Prefer boring solutions over clever hacks

---

### Learning and Growth

- **Continuous learning**: Technology evolves; we must evolve with it
- **Share knowledge**: Document what you learn; mentor others
- **Experiment safely**: Use feature flags and A/B tests for experiments
- **Retrospectives**: Regularly reflect on what works and what doesn't
- **No broken windows**: Fix issues when you see them, don't postpone
- **Boy Scout Rule**: Leave the code cleaner than you found it
- **Single Responsibility**: Each component should have one clear purpose
- **Explicit over implicit**: Be clear about what the code does

### Testing
- **Test coverage is mandatory**: New features require tests
- **Test behavior, not implementation**: Focus on what the code does, not how
- **Edge cases matter**: Test the boundaries and error conditions

### Documentation
- **Document the why, not just the what**: Future developers need context
- **Keep documentation close to the code**: Update docs with code changes
- **SOUL.md is our constitution**: This document guides all development decisions

## Technical Standards

### Backend (Rust)
- **Clippy warnings are errors**: Fix all warnings before merging
- **Error handling**: Use proper error types, avoid unwrap()
- **Database migrations**: Always test migrations on production-like data
- **API design**: RESTful, consistent, versioned

### Frontend (Svelte 5)
- **Svelte 5 runes**: Use $state, $derived, $effect appropriately
- **No complex logic in templates**: Move logic to helpers
- **Accessibility first**: ARIA labels, keyboard navigation, screen reader support
- **Responsive design**: Mobile-first, touch-friendly interfaces

### Infrastructure
- **Docker best practices**: Multi-stage builds, small images
- **Environment separation**: Development, staging, production configs
- **Monitoring**: Health checks, logging, metrics
- **Backup strategy**: Automated, tested, reliable

## Quality Gates

Before any code is merged, it must pass:

### Automated Checks
- [ ] All tests pass (unit, integration, e2e)
- [ ] No new Clippy warnings
- [ ] No new compiler warnings
- [ ] Code coverage maintained or improved
- [ ] Documentation updated
- [ ] Changelog updated

### Manual Review
- [ ] Architecture decisions documented
- [ ] Security implications considered
- [ ] Performance impact assessed
- [ ] Accessibility compliance verified
- [ ] User experience validated

## Development Workflow

1. **Issue first**: Always start with a clear issue or user story
2. **Design discussion**: Architecture decisions require team consensus
3. **Small PRs**: Keep changes focused and reviewable
4. **Code review**: Thorough, constructive, and timely
5. **Testing**: Comprehensive test coverage for all changes
6. **Documentation**: Update all relevant docs
7. **Deployment**: Follow the deployment checklist

## Deployment Principles

### Staging Environment
- Mirror production as closely as possible
- Automated testing before production deployment
- Load testing for new features

### Rollback Strategy
- Every deployment must have a clear rollback plan
- Database migrations must be backward-compatible
- Feature flags for gradual rollouts

## Communication

### Internal
- Daily standups for active contributors
- Architecture discussions in #dev channel
- Code reviews are learning opportunities

### External
- Transparent about issues and timelines
- Responsive to user feedback
- Clear release notes

## Sustainability

### Burnout prevention
- Sustainable pace over crunch time
- Clear boundaries between work and rest
- Encourage time off and disconnection

### Long-term vision
- Invest in tools and automation
- Regular refactoring and debt reduction
- Knowledge sharing and documentation

## This Document

This SOUL.md is a living document. It should evolve with our project and team. Changes require discussion and consensus.

**Last updated**: 2026-05-05
**Next review**: 2026-06-05

## References

- Nook Architecture Decision Records (ADRs)
- Coding Standards and Style Guides
- Testing Strategy Document
- Deployment Checklist