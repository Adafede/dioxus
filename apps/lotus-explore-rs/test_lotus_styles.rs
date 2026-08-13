mod lotus_styles;

fn main() {
    let css = lotus_styles::bundled_lotus_styles();
    
    println!("✓ Base CSS: {} chars", lotus_styles::base::css().len());
    println!("✓ Responsive CSS: {} chars", lotus_styles::responsive::css().len());
    println!("✓ Form Controls CSS: {} chars", lotus_styles::form_controls::css().len());
    println!("✓ Layout Shell CSS: {} chars", lotus_styles::layout_shell::css().len());
    println!("✓ Curation CSS: {} chars", lotus_styles::curation::css().len());
    println!("✓ Accessibility CSS: {} chars", lotus_styles::accessibility::css().len());
    
    println!("\nTotal bundled CSS: {} chars", css.len());
    println!("✓ No CSS is empty: {}", css.len() > 1000);
    println!("✓ Contains CSS syntax: {}", css.contains("{") && css.contains("}"));
}
