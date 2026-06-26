---
name: database-migration-debugger
description: Systematic approach to diagnose and fix database migration issues, particularly FOREIGN KEY constraint failures and migration order conflicts
version: 1.0.0
author: Hermes Agent
license: MIT
tags: [database, migrations, sql, foreign-key, troubleshooting]
related_skills: [systematic-debugging, test-driven-development]
---

# Database Migration Debugger

## Overview
Systematic approach to diagnose and fix database migration issues, particularly FOREIGN KEY constraint failures and migration order conflicts.

## When to Use
- Migration fails with FOREIGN KEY constraint errors
- Database schema mismatches between code and migrations
- Conflicts between manual schema fixes and migration scripts
- Migration order issues causing runtime panics

## Prerequisites
- Access to migration files (SQL or code-generated)
- Knowledge of the database structure
- Understanding of the application's DB initialization code

## Step-by-Step Process

### 1. Locate Migration Files
```bash
# Search for migration directories
find /path/to/project -type d -iname "*migration*" 2>/dev/null

# Search for SQL migration files
find /path/to/project -name "*.sql" 2>/dev/null
```

### 2. Analyze Migration Order and Content
- List migrations in numerical/alphabetical order
- Identify the problematic migration (usually the one failing)
- Check for:
  - DROP TABLE statements that might violate FK constraints
  - CREATE TABLE with FOREIGN KEY references
  - ALTER TABLE operations that might conflict with existing schema

### 3. Examine DB Initialization Code
Look for:
- `main.rs` or equivalent entry point
- Database connection setup
- Migration execution logic (`migrate!().run()`)
- Any pre-migration schema fix functions

### 4. Check for Schema Fix Functions
Common patterns:
```rust
async fn fix_events_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Adds missing columns before migrations run
    // May conflict with DROP TABLE in migrations
}
```

### 5. Verify Foreign Key Dependencies
- Search all migration files for FOREIGN KEY references to the problematic table
- Use grep or search_files:
```bash
grep -r "FOREIGN KEY.*table_name" /path/to/migrations/
```

### 6. Identify Conflicts
Typical conflicts:
- `fix_events_schema()` adds columns → Migration tries to DROP and recreate table
- Migration 001 creates table with FK → Later migration tries to DROP it
- Missing tables referenced by FOREIGN KEY (table not created yet)

### 7. Propose Solutions
Based on conflict type:

**A. Schema Fix vs DROP TABLE Conflict**
- Remove the DROP TABLE and use ALTER TABLE instead
- Or remove the schema fix and let migration handle everything
- Ensure FK constraints are properly ordered

**B. Missing Table Reference**
- Check migration order (move FK-dependent migrations after referenced table creation)
- Add conditional checks before creating FK constraints

**C. Data Migration Issues**
- Add data migration steps before schema changes
- Use transactions to rollback on failure

### 8. Validate Changes
- Test migrations on a fresh database
- Verify FK constraints are properly enforced
- Check application code for expected schema

## Common Pitfalls
- **SQLite FOREIGN KEY enforcement**: Must be explicitly enabled (SQLx does this by default)
- **Migration order**: Alphabetical sorting may not match logical dependencies
- **Schema fix functions**: Can interfere with migration logic if not coordinated
- **DROP TABLE with FK references**: SQLite won't allow DROP if other tables reference it

## Example: Nook Events Migration Fix
**Problem**: Migration 016 tries to DROP events table, but fix_events_schema() already added columns.

**Solution**: Modify 016_events.sql to use ALTER TABLE instead of DROP/CREATE when table exists, or remove the fix function and handle schema changes entirely in migrations.

## Tools Used
- `find` for locating files
- `grep` for searching content
- `read_file` for examining file contents
- Database introspection (if accessible)

## Success Criteria
- Migrations apply cleanly to empty database
- No FOREIGN KEY constraint errors on startup
- Application can read/write to all tables
- Schema matches code expectations

## Related Skills
- database-schema-analysis
- rust-sqlx-migration-troubleshooting
- sqlite-foreign-key-configuration