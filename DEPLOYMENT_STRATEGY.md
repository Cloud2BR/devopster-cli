# Deployment Strategy

This document outlines the deployment strategy for DevOpster, using GitHub Pages with branch-based environments.

## Environments

| Branch | Environment | Purpose | URL |
|--------|-------------|---------|-----|
| `main` | **Production** | Stable, production-ready release | `https://cloud2br.github.io/devopster/` |
| `test` | **Testing** | Pre-release, testing, and staging | `https://cloud2br.github.io/devopster/test/` |

## Deployment Flow

### Main Branch (Production)
- **Trigger**: Push to `main` branch
- **Deployment Target**: Root of GitHub Pages
- **URL**: `https://cloud2br.github.io/devopster/`
- **Use Case**: Production-ready releases, stable documentation, and validated features
- **Review**: Requires Pull Request review before merge to main

### Test Branch (Testing)
- **Trigger**: Push to `test` branch
- **Deployment Target**: `/test/` subdirectory of GitHub Pages
- **URL**: `https://cloud2br.github.io/devopster/test/`
- **Use Case**: Pre-release testing, experimental features, and staging validations
- **Review**: Can accept direct pushes or PRs for rapid iteration

## Workflow Details

Both deployments are automated via GitHub Actions:
- Workflow file: `.github/workflows/deploy-pages.yml`
- Triggered on push to either `main` or `test` branch
- Workflow triggers `workflow_dispatch` for manual deployments
- Assets are cached and versioned per deployment

## Version Tracking

Each deployment includes:
- **Release snapshot**: Latest release metadata in `assets/releases/latest.json`
- **Asset versioning**: Timestamped assets for cache busting
- **Branch-specific resources**: Separate asset directories per environment

## Local Testing

To test Pages locally before deployment:
```bash
# Test main branch deployment
cd docs
# Preview using your preferred local HTTP server
python3 -m http.server 8000

# Test test branch deployment
cd docs  
# The /test/ path will be added automatically in CI
```

## Security & Access Control

- **Production (`main`)**: Protected branch with required reviews
- **Testing (`test`)**: Faster iteration, but same credential/token scope
- **Permissions**: Both use the same GitHub Actions token with `pages:write` scope

## Migration Notes

- Old deployments remain in root and won't be cleared
- `/test/` is a new subdirectory; no conflicts with existing production content
- Rollback available via GitHub Actions workflow re-runs or reverting branches
