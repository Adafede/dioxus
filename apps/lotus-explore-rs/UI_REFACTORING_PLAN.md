# LOTUS Explore UI Refactoring Plan

**Audit Date:** 2026-08-12  
**Total Components:** 49 files  
**Total Style Functions:** 136+  
**Focus Areas:** Component Modularity, CSS Extraction, Naming Conventions, Type Safety

---

## Executive Summary

The lotus-explore-rs UI codebase demonstrates **good foundational practices** but has opportunities for improvement in **consistency, modularity, and maintainability**. Key findings:

✅ **Strengths:**
- StyleBuilder is already widely used (reducing raw CSS)
- Components are well-organized with clear directory structure
- Excellent use of context-based state management
- Good separation of concerns (form, results table, layout)
- Performance optimizations (memoization, Arc selectors)

⚠️ **Improvement Opportunities:**
- Mixed CSS class strings and StyleBuilder (inconsistent approach)
- Some large components (505+ lines) that could be split further
- Magic values in styles (colors, sizes, spacing) not consistently extracted
- Naming inconsistencies (some `_style()` functions, some styles inline)
- CSS classes still present alongside generated styles
- Some style functions could be grouped into logical modules
- Document head management not fully Rust-native

---

## Current State Analysis

### Component Structure Overview

```
src/components/
├── results_table/           (19 files) - Complex table virtualization + tooling
├── form_sections/           (5 files)  - Search form inputs
├── layout/                  (8 files)  - Header, footer, sidebar, metadata
├── data_curation_page/      (3 files)  - Curation workflow page
├── search_panel.rs          (447 lines)
├── results_viewport.rs      (177 lines)
├── curation_results_table.rs(353 lines)
├── form_inputs.rs           (100+ lines)
├── copy_button.rs           (163 lines)
└── loading.rs               (168 lines)

src/lotus_styles/
├── mod.rs                   (bundled CSS injection)
├── base.rs                  (design tokens + resets)
├── form_controls.rs         (raw CSS strings for forms)
├── layout_shell.rs
├── responsive.rs
├── accessibility.rs
└── curation.rs
```

### Style Generation Patterns

**Pattern 1: StyleBuilder (✓ Preferred)**
```rust
fn button_base_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .border("1px solid var(--border)")
        .build()
}
```
Used in: `loading.rs`, `query_panel.rs`, `copy_button.rs` ✓

**Pattern 2: CSS Classes (⚠️ Mixed)**
```rust
// Still using CSS classes from lotus_styles/form_controls.rs
class: "search-panel"
class: "form-section"
class: "search-btn"
```
Issues: Relies on bundled CSS; harder to track dependencies; mixes approaches.

**Pattern 3: Inline Styles (Rare)**
- Generally avoided, which is good
- When used, it's minimal

### CSS Class Usage (Classes to Migrate)

Based on grep analysis, CSS classes still in use:
- `.search-panel` (search_panel.rs)
- `.search-panel-body` (search_panel.rs)
- `.search-btn` (search_panel.rs)
- `.stat-bar` (stat_bar.rs)
- `.meta-item`, `.meta-key`, `.meta-sep`, `.meta-val`, `.mono` (header_meta.rs)
- `.form-section`, `.form-label`, `.form-input`, `.range-inputs` (form_controls.rs)
- Form-related classes in form_sections/basic_sections.rs

### Magic Values Currently in Use

**Colors:**
```rust
stripe_color: "var(--wd-compound-stripe)"
stripe_color: "var(--wd-taxon-stripe)"
stripe_color: "var(--wd-reference-stripe)"
stripe_color: "var(--wd-entries-stripe)"
```

**Sizes:**
```rust
const ROW_HEIGHT_PX_COMFORTABLE: usize = 114;
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
const TABLE_VIEWPORT_FALLBACK_PX: usize = 640;
```

**Spacing in styles:**
```rust
padding("2px 8px")        // button_xs_style
padding("48px")           // loading_state_style
gap("14px")               // stat_badge
```

---

## Detailed Findings by Area

### 1. Results Table Component (19 files)

**Current State:**
- Well-organized but complex
- Good: Separate models for virtualization, sorting, headers
- Issue: `stat_bar.rs` still uses CSS classes (`class: "stat-bar"`)
- Issue: `download_actions.rs` (365 lines) mixing logic and rendering
- Issue: Multiple style functions scattered without clear module

**Key Files:**
- `virtualized_table.rs` (100+ lines) - renders table with virtualization
- `stat_bar.rs` (217 lines) - stats display; needs refactor to pure StyleBuilder
- `download_actions.rs` (365 lines) - needs splitting
- `query_panel.rs` (78 lines) - already well styled but could extract chevron logic
- `table_header.rs` (80 lines) - needs style constants

**Specific Issues:**
```rust
// stat_bar.rs line 74 - CSS class
class: "stat-bar",
style: stat_bar_style(),  // Mixed approach!

// Should be pure style
```

### 2. Form Components (5 files + form_inputs.rs)

**Current State:**
- `form_sections/basic_sections.rs` (236 lines) - well modularized
- Uses CSS classes from lotus_styles (`.form-section`, `.form-label`)
- Style functions exist but CSS class approach is inconsistent

**Issues:**
```rust
// basic_sections.rs - mixing classes and styles
div { style: section_card_style(),
    label { style: label_base_style(), ... }
    input { style: input_base_style(), ... }
}

// But also relies on bundled CSS classes for form-section styling
// causing dependency on external CSS file
```

**Opportunity:**
- Convert all `.form-section`, `.form-label`, `.form-input` to StyleBuilder
- Extract magic values like `12px`, `8px` gaps to constants

### 3. Layout Components (8 files)

**Current State:**
- `header_meta.rs` (100+ lines) - good modularity but uses CSS classes
- `footer.rs` (228 lines) - well-styled with StyleBuilder
- `page_header.rs` - needs review
- `layout_shell.rs` - CSS bundle file

**Specific Issues:**
```rust
// header_meta.rs - still using CSS classes
span { class: "meta-item",
    span { class: "meta-key", ... }
    span { class: "meta-sep", ":" }
    span { class: "meta-val mono", ... }
}

// Should be:
span { style: meta_item_style(),
    span { style: meta_key_style(), ... }
    span { style: meta_separator_style(), ":" }
    span { style: meta_value_style(), ... }
}
```

### 4. Document Head (document_head.rs)

**Current State:**
- Uses `bundled_lotus_styles()` - hybrid Rust + CSS approach
- LinkSpec constants are well-organized
- JSON-LD is correctly handled as Rust string

**Opportunity:**
- Migrate bundled CSS modules to direct Rust generation
- Create style modules for each logical area (form_controls.rs, layout_shell.rs, etc.)

---

## Refactoring Priority Matrix

### Phase 1: Quick Wins (3-5 days, 1-2 files)
**Impact:** HIGH | Effort: LOW | Risk: MINIMAL

1. **Create `ui/style_constants.rs`** - Centralize all magic values
   - Color tokens (stripes, accents, borders)
   - Size tokens (spacing, padding, margins)
   - Typography tokens (font sizes, weights)
   - Estimated: 2-3 hours

2. **Migrate `stat_bar.rs` to pure StyleBuilder**
   - Remove `class: "stat-bar"`
   - Extract `stat_badge_style()` components
   - Add constants for stripe colors
   - Estimated: 2 hours

3. **Extract `query_panel.rs` style variants**
   - Create `query_summary_chevron_open()`, `closed()` constants
   - Consolidate gradient/transition duplicates
   - Estimated: 1 hour

**Expected Outcome:**
- Established pattern for style constants
- Zero CSS class dependencies in 3 key components
- ~50 lines of duplicated code consolidated

---

### Phase 2: Medium Effort (1-2 weeks, 5-10 files)
**Impact:** HIGH | Effort: MEDIUM | Risk: LOW

1. **Refactor Form Components (`form_sections/`, `form_inputs.rs`)**
   - Convert `.form-section` → `section_card_style()` (already exists)
   - Convert `.form-label` → style functions with variants (base, small, uppercase)
   - Convert `.form-input` → `input_base_style()` (already exists)
   - Extract form spacing constants
   - **Files:** `basic_sections.rs`, `formula_section.rs`, `form_inputs.rs`
   - **Estimated:** 4-5 hours

2. **Split `download_actions.rs` (365 lines)**
   - Extract `DownloadButton` component
   - Extract `DownloadMenu` component
   - Extract style functions to `download_actions_styles.rs`
   - **Estimated:** 3-4 hours

3. **Migrate `header_meta.rs` CSS classes**
   - Remove `.meta-item`, `.meta-key`, `.meta-sep`, `.meta-val` classes
   - Create pure Rust components: `MetaItem`, `MetaValue`, `MetaSeparator`
   - **Estimated:** 2 hours

4. **Refactor `layout/notices.rs` (179 lines)**
   - Extract notice-specific styles
   - Create `NoticeBar` component if not already extracted
   - **Estimated:** 2-3 hours

**Expected Outcome:**
- 50+ lines of bundled CSS converted to Rust
- Form styling fully consistent
- 3-5 new sub-components created
- Easier to test and maintain

---

### Phase 3: Larger Refactoring (2-3 weeks, Architectural)
**Impact:** MEDIUM | Effort: HIGH | Risk: MEDIUM

1. **Convert CSS bundles to Rust modules**
   - Replace `lotus_styles/form_controls.rs` raw CSS strings with Rust generators
   - Consolidate form styling into `components/form_sections/styles.rs`
   - Each logical area gets its own module
   - **Estimated:** 5-6 hours

2. **Split large result table components**
   - `virtualized_table.rs` → separate `RowRenderer`, `VirtualizationConfig`
   - `sort_model.rs` (445 lines) → Extract comparison logic
   - Create dedicated module for table cell renderers
   - **Estimated:** 4-5 hours

3. **Extract document head as structured Rust**
   - Replace bundled CSS strings with modular generators
   - Move inline scripts to dedicated functions
   - **Estimated:** 3 hours

4. **Create UI component library module**
   - Extract reusable patterns: `Badge`, `Card`, `Section`, `Panel`
   - These become building blocks for future components
   - **Estimated:** 4-5 hours

**Expected Outcome:**
- No external CSS dependencies (all Rust-generated)
- Established patterns for common UI patterns
- Better reusability and consistency
- Easier onboarding for new contributors

---

## Code Examples: Before & After

### Example 1: StatBadge Component (Before → After)

**BEFORE** (stat_bar.rs line 36-48):
```rust
#[component]
fn StatBadge(
    value: usize,
    secondary_value: Option<usize>,
    secondary_label: Option<&'static str>,
    noun: CountNoun,
    plus: bool,
    stripe_color: &'static str,
) -> Element {
    rsx! {
        div { style: stat_badge_style(stripe_color),  // Color parameter
            div { style: stat_value_row_style(),
                span { style: stat_value_style(), "{display_value}" }
                if let Some(secondary_text) = secondary_inline.as_ref() {
                    span { style: stat_secondary_style(), "{secondary_text}" }
                }
            }
            span { style: stat_label_style(), "{label}" }
        }
    }
}
```

**AFTER** (with constants):
```rust
// ui/style_constants.rs
pub mod stat_stripe_colors {
    pub const COMPOUND: &str = "var(--wd-compound-stripe)";
    pub const TAXON: &str = "var(--wd-taxon-stripe)";
    pub const REFERENCE: &str = "var(--wd-reference-stripe)";
    pub const ENTRIES: &str = "var(--wd-entries-stripe)";
}

pub mod stat_spacing {
    pub const GAP: &str = "14px";
    pub const VALUE_FONT_SIZE: &str = "var(--fs-stat)";
}

// stat_bar.rs
#[component]
fn StatBadge(
    value: usize,
    secondary_value: Option<usize>,
    secondary_label: Option<&'static str>,
    noun: CountNoun,
    plus: bool,
    stripe: StatStripe,  // Enum instead of string!
) -> Element {
    let stripe_color = match stripe {
        StatStripe::Compound => stat_stripe_colors::COMPOUND,
        StatStripe::Taxon => stat_stripe_colors::TAXON,
        StatStripe::Reference => stat_stripe_colors::REFERENCE,
        StatStripe::Entries => stat_stripe_colors::ENTRIES,
    };
    // ... rest of component
}
```

**Benefits:**
- Type-safe stripe selection (no strings)
- Centralized color definitions
- Easy to audit/change all stripes at once
- IDE autocomplete for all options

---

### Example 2: Form Section Styling (Before → After)

**BEFORE** (basic_sections.rs line 22-43):
```rust
rsx! {
    div { style: section_card_style(),  // Rust style
        label { style: label_base_style(), r#for: "taxon-input", ... }
        input {
            style: input_base_style(),  // Rust style
            // ...
        }
        p { style: hint_text_style(), ... }
    }
}

// But form-section CSS in lotus_styles/form_controls.rs depends on:
// .form-section { display:flex; ... border:1px solid var(--panel-border); ... }
// Creates dependency on bundled CSS!
```

**AFTER** (with full Rust styles):
```rust
// components/form_sections/styles.rs
pub fn form_section_card() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("5px")
        .padding(spacing::FORM_SECTION)  // "10px 12px"
        .border("1px solid var(--panel-border)")
        .border_radius("12px")
        .background_color("var(--panel-bg-soft)")
        .build()
}

pub fn form_section_nested() -> String {
    StyleBuilder::new()
        .property("padding-left", spacing::NESTED_SECTION)  // "10px"
        .property("border-left", "1px solid var(--border)")
        .property("margin-top", "4px")
        .build()
}

// basic_sections.rs
rsx! {
    div { style: form_section_card(),  // No external CSS dependency!
        label { style: label_base_style(), r#for: "taxon-input", ... }
        input { style: input_base_style(), ... }
        p { style: hint_text_style(), ... }
    }
}
```

**Benefits:**
- Self-contained styling (no external CSS)
- Easy to find all usages
- Type-safe via Rust functions
- Easier to test components in isolation

---

### Example 3: Component Splitting (Before → After)

**BEFORE** (download_actions.rs - 365 lines):
```rust
pub fn DownloadActionsGroup() -> Element {
    // 50 lines of setup
    // 200 lines of button rendering with inline closures
    // 100 lines of download dispatch logic mixed in
    rsx! {
        div { style: toolbar_actions_style(),
            button { style: button_base_style(),
                onclick: move |_| {
                    *download_busy.write() = true;
                    spawn(/* ... download logic ... */);
                },
                // ...
            }
            // More buttons...
        }
    }
}
```

**AFTER** (split components):
```rust
// components/results_table/table_toolbar_sections/download_actions_components.rs
#[component]
fn DownloadButton(
    label: &'static str,
    on_click: EventHandler<()>,
    is_busy: bool,
) -> Element {
    rsx! {
        button {
            style: download_button_style(),
            disabled: is_busy,
            onclick: move |_| on_click.call(()),
            if is_busy {
                span { style: spinner_sm_style(), "aria-hidden": "true" }
            }
            "{label}"
        }
    }
}

#[component]
fn DownloadMenu(
    on_csv: EventHandler<()>,
    on_json: EventHandler<()>,
    on_rdf: EventHandler<()>,
    is_busy: bool,
) -> Element {
    rsx! {
        div { style: download_menu_style(),
            DownloadButton { label: "CSV", on_click: on_csv, is_busy }
            DownloadButton { label: "JSON", on_click: on_json, is_busy }
            DownloadButton { label: "RDF", on_click: on_rdf, is_busy }
        }
    }
}

pub fn DownloadActionsGroup() -> Element {
    // 50 lines of setup
    // Clear business logic at top
    let on_csv = move |_| dispatch_download(Format::CSV, ...);
    
    rsx! {
        div { style: toolbar_actions_style(),
            DownloadMenu {
                on_csv,
                on_json,
                on_rdf,
                is_busy: *download_busy.read(),
            }
        }
    }
}
```

**Benefits:**
- Reusable components (`DownloadButton`, `DownloadMenu`)
- Easier to test (can test button in isolation)
- Clearer data flow
- Easier to add new download formats

---

## Naming Conventions & Standards

### Established Patterns to Maintain

✓ **Component names:** PascalCase + descriptive (`StatBadge`, `CopyButton`, `QueryPanel`)
✓ **Style functions:** snake_case + `_style()` suffix (`button_base_style()`, `stat_badge_style()`)
✓ **Constants:** UPPER_SNAKE_CASE for sizes/counts (`ROW_HEIGHT_PX_COMFORTABLE`, `VIRTUAL_OVERSCAN_ROWS`)
✓ **Module structure:** Clear hierarchy (components/form_sections/basic_sections.rs)

### New Standards to Introduce

**Style Constants Module:**
```rust
// ui/style_constants.rs
pub mod spacing {
    pub const FORM_SECTION: &str = "10px 12px";
    pub const BUTTON_XS_PAD: &str = "2px 8px";
}

pub mod colors {
    pub mod stat_stripes {
        pub const COMPOUND: &str = "var(--wd-compound-stripe)";
        // ...
    }
}

pub mod dimensions {
    pub const SPINNER_LG_SIZE: &str = "40px";
    pub const BUTTON_MIN_HEIGHT: &str = "40px";
}
```

**Style Modules per Logical Area:**
```rust
// components/form_sections/styles.rs
pub fn form_section_card() -> String { ... }
pub fn label_base() -> String { ... }
pub fn label_small() -> String { ... }

// components/results_table/table_toolbar_sections/styles.rs
pub fn stat_badge(stripe: &str) -> String { ... }
pub fn stat_value() -> String { ... }
```

**Component Sub-modules:**
```rust
// components/form_sections/basic_sections.rs
mod styles;
use styles::*;

// components/form_sections/formula_section.rs
mod styles;
use styles::*;
```

---

## Implementation Roadmap

### Week 1: Foundation
- [ ] Create `ui/style_constants.rs` with all magic values
- [ ] Create `StatStripe` enum and color constants
- [ ] Audit all component style functions (document in spreadsheet)
- [ ] Set up style module structure (one per component area)
- **Deliverable:** Constants library, audit report

### Week 2-3: Phase 1 & 2 Quick Wins
- [ ] Migrate `stat_bar.rs` → pure StyleBuilder
- [ ] Migrate `query_panel.rs` → extract constants
- [ ] Refactor `form_sections/` → pure Rust styles
- [ ] Split `download_actions.rs` → sub-components
- [ ] Migrate `header_meta.rs` → remove CSS classes
- **Deliverable:** 5-8 components fully refactored

### Week 4-5: Phase 2 Medium Effort
- [ ] Convert `lotus_styles/form_controls.rs` → Rust module
- [ ] Extract notice/alert patterns
- [ ] Create reusable component library
- [ ] Full test coverage on new components
- **Deliverable:** 100% CSS class removal from components

### Week 6-7: Phase 3 Architectural
- [ ] Convert remaining bundled CSS to Rust
- [ ] Split large table components
- [ ] Refactor document_head.rs
- [ ] Create UI patterns library
- **Deliverable:** Zero external CSS dependencies, established patterns

### Week 8: Polish & Documentation
- [ ] Comprehensive style guide update
- [ ] Code examples for new contributors
- [ ] Performance audit (ensure no regressions)
- [ ] Update component documentation
- **Deliverable:** Published style guide, performance report

---

## Metrics & Success Criteria

### Code Quality Metrics

| Metric | Current | Target | Weight |
|--------|---------|--------|--------|
| CSS classes in components | ~15 | 0 | HIGH |
| StyleBuilder consistency | 60% | 100% | HIGH |
| Style functions per file | 3-5 | 2-4 | MEDIUM |
| Magic values in code | ~40 | 0 | HIGH |
| Avg component size (lines) | 240 | <200 | MEDIUM |
| Test coverage for styles | 0% | 60%+ | MEDIUM |

### Maintainability Improvements

- **Single Responsibility:** Each style function has one purpose
- **Reusability:** Common patterns extracted to library
- **Consistency:** All styling follows one approach (StyleBuilder)
- **Discoverability:** Style constants indexed in central location
- **Type Safety:** Magic values replaced with typed constants/enums

### Performance Expectations

✓ **No regressions:** Build times should not increase  
✓ **Bundle size:** Slight reduction from consolidated CSS  
✓ **Runtime:** No changes (StyleBuilder still generates same strings)

---

## Risk Assessment & Mitigation

### Medium Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Missing CSS class dependencies | Medium | High | Audit bundled CSS before refactoring, comprehensive grep |
| Breaking responsive behavior | Low | High | Full cross-browser testing after each phase |
| Performance regression | Low | Medium | Benchmark before/after builds and runtime |

### Mitigation Strategies

1. **Incremental rollout:** One component area at a time with full testing
2. **Comprehensive testing:** UI tests for all visual changes
3. **Documentation:** Document all style decisions and patterns
4. **Backup branches:** Maintain clear git history for easy rollback

---

## Recommendations

### Do First (High Priority)

1. ✅ Create `ui/style_constants.rs` immediately (enables all other work)
2. ✅ Document all current CSS class dependencies (grep audit)
3. ✅ Establish `StatStripe` enum (unifies color selection)
4. ✅ Create style module per component area (organize existing functions)

### Do Soon (Medium Priority)

5. Migrate form components (highest CSS class usage)
6. Split `download_actions.rs` (largest UI component)
7. Establish component library patterns

### Do Later (Lower Priority)

8. Convert bundled CSS to Rust generators (architectural, lower ROI)
9. Split large table components (already well-optimized)
10. Create comprehensive style documentation

### Avoid

- Don't mix CSS classes and StyleBuilder in same component
- Don't create style functions without centralizing constants
- Don't skip tests for UI changes
- Don't refactor without clear PR scope

---

## References & Related Code

**Key Files to Review:**
- `src/components/copy_button.rs` - Good style function example
- `src/components/loading.rs` - Excellent pure StyleBuilder usage
- `src/components/form_sections/basic_sections.rs` - Mixed approach (needs refactoring)
- `src/lotus_styles/base.rs` - Design tokens (migrate to Rust)
- `src/document_head.rs` - Hybrid approach (needs refactoring)

**Dependencies:**
- `ui::prelude::*` - Contains `StyleBuilder`
- `dioxus::prelude::*` - Component framework
- Design tokens in `:root` CSS (need Rust equivalents)

**Related Documentation:**
- [StyleBuilder Documentation](https://github.com/DioxusLabs/dioxus)
- [Dioxus Component Patterns](https://dioxuslabs.com/)
- Current LOTUS style guide (if exists)

---

## Questions for Stakeholder Review

1. **Component library:** Should we create a separate UI library crate (e.g., `lotus-ui`) or keep it in-app?
2. **CSS classes:** Any legacy compatibility requirements for CSS classes?
3. **Performance:** Are there performance budgets we should respect?
4. **Responsive design:** Priority on mobile/tablet optimization?
5. **Accessibility:** Additional a11y requirements beyond current ARIA labels?
6. **Timeline:** Preferred timeline for phases (weeks vs. months)?
7. **Team capacity:** Developer time available per week?

---

## Appendix: File-by-File Status

### ✅ Already Well-Styled (Minimal Changes)
- `copy_button.rs` - Pure StyleBuilder
- `loading.rs` - Pure StyleBuilder  
- `layout/footer.rs` - Pure StyleBuilder
- `results_table/query_panel.rs` - Mostly StyleBuilder
- `results_table/table_header.rs` - Mostly StyleBuilder

### ⚠️ Mixed/Needs Refactoring (Medium Effort)
- `search_panel.rs` - Mix of classes and styles
- `form_sections/basic_sections.rs` - Mix of classes and styles
- `layout/header_meta.rs` - Heavy CSS class usage
- `results_table/stat_bar.rs` - Mix of classes and styles
- `results_table/download_actions.rs` - Large, needs splitting

### 🔴 Heavily CSS-Dependent (High Effort)
- `lotus_styles/form_controls.rs` - Raw CSS strings
- `lotus_styles/layout_shell.rs` - Raw CSS strings
- `document_head.rs` - Bundled CSS injection
- `data_curation_page/sections.rs` - Check CSS dependencies

---

**Plan prepared for review and approval before implementation begins.**

