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

### 💼 Other

- Update release workflow

### 📚 Documentation

- README

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

### 💼 Other

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

### 💼 Other

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

### 💼 Other

- Dockerize application and create github workflow

### 🎨 Styling

- *(backend)* Format code

### ◀️ Revert

- No target_host_id in application_domain junction table
