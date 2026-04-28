## [1.4.5] - 2026-04-28

### 🐛 Bug Fixes

- *(backend + frontend)* Don't limit healthcheck time, but count by [@mielpeeters](https://github.com/mielpeeters)
## [1.4.4] - 2026-04-28

### 🐛 Bug Fixes

- *(backend)* Load healthcheck results from last hour on startup by [@mielpeeters](https://github.com/mielpeeters)
## [1.4.3] - 2026-04-28

### 🚀 Features

- *(backend)* Split HOST to HOST and PORT by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Title auto -> Auto by [@mielpeeters](https://github.com/mielpeeters)

### 🔨 Build

- Build frontend with cargo build.rs by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Update README by [@mielpeeters](https://github.com/mielpeeters)
- Update dev.env by [@mielpeeters](https://github.com/mielpeeters)
- Simplify justfile by [@mielpeeters](https://github.com/mielpeeters)
## [1.4.2] - 2026-04-27

### 🚀 Features

- *(frontend)* Contribution types better suit GhentCDH now by [@mielpeeters](https://github.com/mielpeeters)
## [1.4.1] - 2026-03-31

### 🚀 Features

- *(backend)* Better outline sync formatting by [@mielpeeters](https://github.com/mielpeeters)

### 📚 Documentation

- Update openapi documentation by [@mielpeeters](https://github.com/mielpeeters)

### 🎨 Styling

- Reformat (cargo fmt) by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Prek.toml by [@mielpeeters](https://github.com/mielpeeters)
- Script for checking if openapi documentation is in sync by [@mielpeeters](https://github.com/mielpeeters)
- Add openapi-in-sync as prek hook ([#114](https://github.com/GhentCDH/auto/pull/114)) by [@mielpeeters](https://github.com/mielpeeters)
## [1.4.0] - 2026-03-30

### 🚀 Features

- Lower default retry interval and timeout seconds by [@mielpeeters](https://github.com/mielpeeters)
- Create and link favicon ([#104](https://github.com/GhentCDH/auto/pull/104)) by [@mielpeeters](https://github.com/mielpeeters)
- Better visualization and default value for domain creation target by [@mielpeeters](https://github.com/mielpeeters)
- Show selector form by default ([#107](https://github.com/GhentCDH/auto/pull/107)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add '/' keybind to focus global search by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add Ctrl+Enter keybind to submit forms in modals by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add 'e' keybind to open edit modal on detail pages ([#112](https://github.com/GhentCDH/auto/pull/112)) by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Domains card separator has proper width ([#105](https://github.com/GhentCDH/auto/pull/105)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Remove threejs and tresjs ([#108](https://github.com/GhentCDH/auto/pull/108)) by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Reject domain names containing protocol prefix by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Validate domain name has no protocol in form ([#109](https://github.com/GhentCDH/auto/pull/109)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Fix 'Add Storag' typo ([#110](https://github.com/GhentCDH/auto/pull/110)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Hide vertical scrollbar on services table in application detail ([#111](https://github.com/GhentCDH/auto/pull/111)) by [@mielpeeters](https://github.com/mielpeeters)

### 🔨 Build

- Add PR information to git-cliff generated changelogs by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Explicitely scoped cache by [@mielpeeters](https://github.com/mielpeeters)
## [1.3.1] - 2026-02-24

### 🚀 Features

- *(backend)* Better marks for outline outputs, specifying id by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Split domain to HOST and BASE_URL by [@mielpeeters](https://github.com/mielpeeters)
## [1.3.0] - 2026-02-24

### 🚀 Features

- *(frontend)* App and serv duplication by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Overview rust module by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Overview trait and overview endpoints by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Generic resolve endpoints, with id prefix support by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Generic UUID(prefix) resolver by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Add outline_url column to applications and services by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Outline_url columns by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Outline syncing by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Outline syncing by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Outline document <auto> </auto> markers with redundancy by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Outline -> Docs in detail view ([#87](https://github.com/GhentCDH/auto/pull/87)) by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Typechecking problems ([#84](https://github.com/GhentCDH/auto/pull/84)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Breadcrumb shows 'People' insead of 'Persons' or 'Peoples' by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- *(justfile)* Reduce debug logging for verbose crates by [@mielpeeters](https://github.com/mielpeeters)
## [1.2.2] - 2026-02-23

### 🚀 Features

- *(backend)* Sort case-insensitively by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Toast messages ([#81](https://github.com/GhentCDH/auto/pull/81)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Dashboard layout improvements ([#82](https://github.com/GhentCDH/auto/pull/82)) by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Kuma_id is undefined by default for a new healthcheck by [@mielpeeters](https://github.com/mielpeeters)
## [1.2.1] - 2026-02-23

### 🚀 Features

- *(frontend)* Add yellow for retried requests ([#79](https://github.com/GhentCDH/auto/pull/79)) by [@mielpeeters](https://github.com/mielpeeters)
## [1.2.0] - 2026-02-20

### 🚀 Features

- *(frontend)* Allow multi-selecting tech stack badges ([#74](https://github.com/GhentCDH/auto/pull/74)) by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Add endpoint for getting kuma url by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Clicking kuma id goes to kuma url in new tab ([#75](https://github.com/GhentCDH/auto/pull/75)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Healthcheck form UX improvements by [@mielpeeters](https://github.com/mielpeeters)
- Update AGENTS.md ([#76](https://github.com/GhentCDH/auto/pull/76)) by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Update dashboard endpoint to return more information by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Update dashboard ([#77](https://github.com/GhentCDH/auto/pull/77)) by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(backend)* Close socketIO connections before reconnecting new one ([#72](https://github.com/GhentCDH/auto/pull/72)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Image ref doesn't need to be a url ([#73](https://github.com/GhentCDH/auto/pull/73)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Ts errors by [@mielpeeters](https://github.com/mielpeeters)
## [1.1.0] - 2026-02-19

### 🚀 Features

- *(frontend)* Reorder the application detail cards by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Remove max width for container, just use tailwindcss .container by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Custom container class for wider container ([#44](https://github.com/GhentCDH/auto/pull/44)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Allow creating healthchecks from application page by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Clean up application/service healthcheck link by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Allow creating healthchecks from service details page ([#47](https://github.com/GhentCDH/auto/pull/47)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add edit and unlink healthcheck buttons by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Status badges second to last by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Env badges align vertically above eachother ([#48](https://github.com/GhentCDH/auto/pull/48)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Reusable composable useRelationManager ([#49](https://github.com/GhentCDH/auto/pull/49)) by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Add retry, request_body and http_auth columns to healthcheck table by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Use new healthcheck columns by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Update kuma_export.py script by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Show and edit new healthcheck columns ([#50](https://github.com/GhentCDH/auto/pull/50)) by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Image references by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Image references by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Image references by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Store kuma_id in healthcheck by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Handle kuma_id in healthcheck service by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Store interval between healthchecks by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Handle interval in healthcheck service by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Healthcheck service and model updates by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Kuma sync functions, errors, and config by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Sync kuma button by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Script reads interval by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* SSE endpoint for kuma polled uptime information by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Handle uptime SSE and plot results ([#55](https://github.com/GhentCDH/auto/pull/55)) by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Sync one healthcheck to kuma endpoint by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Update healthchecksApi for single kuma sync by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Update component for single healthcheck kuma sync ([#57](https://github.com/GhentCDH/auto/pull/57)) by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Openapi documentation at /api/docs ([#59](https://github.com/GhentCDH/auto/pull/59)) by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Add kuma_dirty column to healthcheck table by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Track kuma sync dirty state on healthchecks by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Show kuma sync dirty state indicators ([#60](https://github.com/GhentCDH/auto/pull/60)) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Auto-expand request body editor for POST methods ([#61](https://github.com/GhentCDH/auto/pull/61)) by [@mielpeeters](https://github.com/mielpeeters)
- Improve global search with healthchecks and cross-entity matching ([#63](https://github.com/GhentCDH/auto/pull/63)) by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Actually remove headers when removing all by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Custom kuma-api implementation that _works_ by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Actually set the Kuma-created monitor id in local table by [@mielpeeters](https://github.com/mielpeeters)

### ◀️ Revert

- *(frontend)* Reusable composable didn't save many loc by [@mielpeeters](https://github.com/mielpeeters)
## [1.0.1] - 2026-02-05

### 🚀 Features

- Add push_to_kuma.py script by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(backend)* Global search uses fqdn, not name by [@mielpeeters](https://github.com/mielpeeters)
## [1.0.0] - 2026-02-05

### 🚀 Features

- *(database)* Add healthcheck table by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Add healthcheck logic by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add healthcheck UIs by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add kuma import wizard by [@mielpeeters](https://github.com/mielpeeters)
## [0.1.2] - 2026-02-03

### 🚀 Features

- *(frontend)* Make EntitySelector tab-navigatable by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Relocate 'Reset filters' button to top by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* New domain table and application relation by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Handle new domain table columns by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Update domain-related pages to new format ([#33](https://github.com/GhentCDH/auto/pull/33)) by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- Workflow release body from git-cliff by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Start_date <= end_date validation by [@mielpeeters](https://github.com/mielpeeters)

### 📚 Documentation

- Write CHANGELOG.md by [@mielpeeters](https://github.com/mielpeeters)

### 🎨 Styling

- *(backend, frontend)* Refactor by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Update cliff.toml config to include build-related commits by [@mielpeeters](https://github.com/mielpeeters)
## [0.1.1] - 2026-01-29

### 🐛 Bug Fixes

- Fix changelog generation in workflow by [@mielpeeters](https://github.com/mielpeeters)
## [0.1.0] - 2026-01-29

### 🚀 Features

- *(database)* Application name and environment are unique by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Service unique on (name, environment) by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Show environment next to name if available by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Improve search with indexes and concurrency by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Drop domain ssl fields by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Don't handle domain ssl fields by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Don't show domain ssl fields by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Form improvements: autofocus and autofill easy fields by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Fake 3d model with layered transparent images by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Cache bounding rect for mascot by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Filters for entitylists by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Implement filtered lists by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Link infra if domain points to it and it isn't linked already by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Align edit & unlink buttons right by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Fill in initial search name for infra form by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Show infrastructure target as clickable entity in domain list by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Autofill ghent email only when name not empty by [@mielpeeters](https://github.com/mielpeeters)

### 📚 Documentation

- README by [@mielpeeters](https://github.com/mielpeeters)

### 🔨 Build

- Update release workflow by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Update justfile by [@mielpeeters](https://github.com/mielpeeters)
- Version bump by [@mielpeeters](https://github.com/mielpeeters)
- Add git-cliff config by [@mielpeeters](https://github.com/mielpeeters)
## [0.0.5] - 2026-01-28

### 🚀 Features

- AUTO lighting update by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Add stacks to global search results by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Show stack global search results by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* EntitySelector with immediate search by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Modals escape key to close and auto-focus search field for EntitySelector by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Migrations for services & infra by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Add services & infrastructure by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Visualizing services & infra by [@mielpeeters](https://github.com/mielpeeters)
- Show version of app in footer by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Fix EntitySelector view for many items by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Unused table headers by [@mielpeeters](https://github.com/mielpeeters)

### 🔨 Build

- *(frontend)* Split large js libraries in separately built chunks by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Version bump by [@mielpeeters](https://github.com/mielpeeters)
## [0.0.4] - 2026-01-27

### 🚀 Features

- *(frontend)* Add AUTO mascot :) by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- Version bump by [@mielpeeters](https://github.com/mielpeeters)
## [0.0.3] - 2026-01-27

### 🚀 Features

- Allow custom values for most select entries by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Technology stack support by [@mielpeeters](https://github.com/mielpeeters)
- *(database)* Stack table migration by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Technology stack support by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Same rounding for light and dark theme by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Stack list page by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Technology stack item details page by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Stack badge is clickable on its name with property by [@mielpeeters](https://github.com/mielpeeters)

### 🐛 Bug Fixes

- *(frontend)* Remove unnecessary type imports from list pages by [@mielpeeters](https://github.com/mielpeeters)

### 🚜 Refactor

- *(frontend)* Extract EntityList from List views by [@mielpeeters](https://github.com/mielpeeters)

### ⚙️ Miscellaneous Tasks

- README by [@mielpeeters](https://github.com/mielpeeters)
- Update image in README by [@mielpeeters](https://github.com/mielpeeters)
- Version update to v0.0.3 by [@mielpeeters](https://github.com/mielpeeters)
## [0.0.2] - 2026-01-27

### 🔨 Build

- Add github workflow docker buildx caching by [@mielpeeters](https://github.com/mielpeeters)
## [0.0.1] - 2026-01-27

### 🚀 Features

- Rust + axum + sqlx + vue + tailwindcss project scaffolding by [@mielpeeters](https://github.com/mielpeeters)
- Add main data structures, CRUD implemenatations and UI by [@mielpeeters](https://github.com/mielpeeters)
- Extract select options into objects by [@mielpeeters](https://github.com/mielpeeters)
- Remove SSL certificate expiration pane in dashboard by [@mielpeeters](https://github.com/mielpeeters)
- *(backend)* Add target_host_id field to domain-application junction table by [@mielpeeters](https://github.com/mielpeeters)
- Domain link can refer to an existing Host by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Edit buttons by [@mielpeeters](https://github.com/mielpeeters)
- *(frontend)* Add and edit notes by [@mielpeeters](https://github.com/mielpeeters)
- Get rid of 'client' by [@mielpeeters](https://github.com/mielpeeters)

### ◀️ Revert

- No target_host_id in application_domain junction table by [@mielpeeters](https://github.com/mielpeeters)

### 🎨 Styling

- *(backend)* Format code by [@mielpeeters](https://github.com/mielpeeters)

### 🔨 Build

- Dockerize application and create github workflow by [@mielpeeters](https://github.com/mielpeeters)
