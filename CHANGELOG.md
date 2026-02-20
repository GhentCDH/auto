## [1.2.0] - 2026-02-20

### 🚀 Features

- *(frontend)* Allow multi-selecting tech stack badges
- *(backend)* Add endpoint for getting kuma url
- *(frontend)* Clicking kuma id goes to kuma url in new tab
- *(frontend)* Healthcheck form UX improvements
- Update AGENTS.md
- *(backend)* Update dashboard endpoint to return more information
- *(frontend)* Update dashboard

### 🐛 Bug Fixes

- *(backend)* Close socketIO connections before reconnecting new one
- *(frontend)* Image ref doesn't need to be a url
- *(frontend)* Ts errors
## [1.1.0] - 2026-02-19

### 🚀 Features

- *(frontend)* Reorder the application detail cards
- *(frontend)* Remove max width for container, just use tailwindcss .container
- *(frontend)* Custom container class for wider container
- *(frontend)* Allow creating healthchecks from application page
- *(backend)* Clean up application/service healthcheck link
- *(frontend)* Allow creating healthchecks from service details page
- *(frontend)* Add edit and unlink healthcheck buttons
- *(frontend)* Status badges second to last
- *(frontend)* Env badges align vertically above eachother
- *(frontend)* Reusable composable useRelationManager
- *(database)* Add retry, request_body and http_auth columns to healthcheck table
- *(backend)* Use new healthcheck columns
- *(backend)* Update kuma_export.py script
- *(frontend)* Show and edit new healthcheck columns
- *(database)* Image references
- *(backend)* Image references
- *(frontend)* Image references
- *(database)* Store kuma_id in healthcheck
- *(backend)* Handle kuma_id in healthcheck service
- *(database)* Store interval between healthchecks
- *(backend)* Handle interval in healthcheck service
- *(backend)* Healthcheck service and model updates
- *(backend)* Kuma sync functions, errors, and config
- *(frontend)* Sync kuma button
- *(backend)* Script reads interval
- *(backend)* SSE endpoint for kuma polled uptime information
- *(frontend)* Handle uptime SSE and plot results
- *(backend)* Sync one healthcheck to kuma endpoint
- *(frontend)* Update healthchecksApi for single kuma sync
- *(frontend)* Update component for single healthcheck kuma sync
- *(backend)* Openapi documentation at /api/docs
- *(database)* Add kuma_dirty column to healthcheck table
- *(backend)* Track kuma sync dirty state on healthchecks
- *(frontend)* Show kuma sync dirty state indicators
- *(frontend)* Auto-expand request body editor for POST methods
- Improve global search with healthchecks and cross-entity matching

### 🐛 Bug Fixes

- *(frontend)* Actually remove headers when removing all
- *(backend)* Custom kuma-api implementation that _works_
- *(backend)* Actually set the Kuma-created monitor id in local table

### ◀️ Revert

- *(frontend)* Reusable composable didn't save many loc
## [1.0.1] - 2026-02-05

### 🚀 Features

- Add push_to_kuma.py script

### 🐛 Bug Fixes

- *(backend)* Global search uses fqdn, not name
## [1.0.0] - 2026-02-05

### 🚀 Features

- *(database)* Add healthcheck table
- *(backend)* Add healthcheck logic
- *(frontend)* Add healthcheck UIs
- *(frontend)* Add kuma import wizard
## [0.1.2] - 2026-02-03

### 🚀 Features

- *(frontend)* Make EntitySelector tab-navigatable
- *(frontend)* Relocate 'Reset filters' button to top
- *(database)* New domain table and application relation
- *(backend)* Handle new domain table columns
- *(frontend)* Update domain-related pages to new format

### 🐛 Bug Fixes

- Workflow release body from git-cliff
- *(frontend)* Start_date <= end_date validation

### 📚 Documentation

- Write CHANGELOG.md

### 🎨 Styling

- *(backend, frontend)* Refactor

### ⚙️ Miscellaneous Tasks

- Update cliff.toml config to include build-related commits
## [0.1.1] - 2026-01-29

### 🐛 Bug Fixes

- Fix changelog generation in workflow
## [0.1.0] - 2026-01-29

### 🚀 Features

- *(database)* Application name and environment are unique
- *(database)* Service unique on (name, environment)
- *(frontend)* Show environment next to name if available
- *(backend)* Improve search with indexes and concurrency
- *(database)* Drop domain ssl fields
- *(backend)* Don't handle domain ssl fields
- *(frontend)* Don't show domain ssl fields
- *(frontend)* Form improvements: autofocus and autofill easy fields
- *(frontend)* Fake 3d model with layered transparent images
- *(frontend)* Cache bounding rect for mascot
- *(frontend)* Filters for entitylists
- *(backend)* Implement filtered lists
- *(backend)* Link infra if domain points to it and it isn't linked already
- *(frontend)* Align edit & unlink buttons right

### 🐛 Bug Fixes

- *(frontend)* Fill in initial search name for infra form
- *(frontend)* Show infrastructure target as clickable entity in domain list
- *(frontend)* Autofill ghent email only when name not empty

### 📚 Documentation

- README

### 🔨 Build

- Update release workflow

### ⚙️ Miscellaneous Tasks

- Update justfile
- Version bump
- Add git-cliff config
## [0.0.5] - 2026-01-28

### 🚀 Features

- AUTO lighting update
- *(backend)* Add stacks to global search results
- *(frontend)* Show stack global search results
- *(frontend)* EntitySelector with immediate search
- *(frontend)* Modals escape key to close and auto-focus search field for EntitySelector
- *(database)* Migrations for services & infra
- *(backend)* Add services & infrastructure
- *(frontend)* Visualizing services & infra
- Show version of app in footer

### 🐛 Bug Fixes

- *(frontend)* Fix EntitySelector view for many items
- *(frontend)* Unused table headers

### 🔨 Build

- *(frontend)* Split large js libraries in separately built chunks

### ⚙️ Miscellaneous Tasks

- Version bump
## [0.0.4] - 2026-01-27

### 🚀 Features

- *(frontend)* Add AUTO mascot :)

### ⚙️ Miscellaneous Tasks

- Version bump
## [0.0.3] - 2026-01-27

### 🚀 Features

- Allow custom values for most select entries
- *(frontend)* Technology stack support
- *(database)* Stack table migration
- *(backend)* Technology stack support
- *(frontend)* Same rounding for light and dark theme
- *(frontend)* Stack list page
- *(frontend)* Technology stack item details page
- *(frontend)* Stack badge is clickable on its name with property

### 🐛 Bug Fixes

- *(frontend)* Remove unnecessary type imports from list pages

### 🚜 Refactor

- *(frontend)* Extract EntityList from List views

### ⚙️ Miscellaneous Tasks

- README
- Update image in README
- Version update to v0.0.3
## [0.0.2] - 2026-01-27

### 🔨 Build

- Add github workflow docker buildx caching
## [0.0.1] - 2026-01-27

### 🚀 Features

- Rust + axum + sqlx + vue + tailwindcss project scaffolding
- Add main data structures, CRUD implemenatations and UI
- Extract select options into objects
- Remove SSL certificate expiration pane in dashboard
- *(backend)* Add target_host_id field to domain-application junction table
- Domain link can refer to an existing Host
- *(frontend)* Edit buttons
- *(frontend)* Add and edit notes
- Get rid of 'client'

### ◀️ Revert

- No target_host_id in application_domain junction table

### 🎨 Styling

- *(backend)* Format code

### 🔨 Build

- Dockerize application and create github workflow
