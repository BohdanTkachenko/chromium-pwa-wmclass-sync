#!/usr/bin/env nu

# Nushell functional test for chromium-pwa-wmclass-sync

def reset_fixtures [apps_dir: string] {
    # Delete everything in the directory
    if (ls $apps_dir | length) > 0 {
        ls $apps_dir | each {|f| rm -rf $f.name }
    }
    
    # Copy fixtures
    cp -r tests/fixtures/* $"($apps_dir)/"
    
    # Ensure writable
    ls $apps_dir | each {|f| chmod +w $f.name }
}

def main [] {
    let apps_dir = (mktemp -d)
    
    print "--- Test Case 1: Without --rename ---"
    reset_fixtures $apps_dir
    chromium-pwa-wmclass-sync --apps-dir $apps_dir -v
    
    print "Verifying results (no rename)..."
    let mismatched = ($apps_dir | path join "mismatched_wmclass.desktop")
    if (not ($mismatched | path exists)) {
        error make {msg: "ERROR: mismatched_wmclass.desktop was unexpectedly renamed"}
    }
    
    let content = (open $mismatched --raw)
    if ($content | str contains "StartupWMClass=wrong-class") {
        error make {msg: "ERROR: StartupWMClass was not fixed"}
    }
    if (not ($content | str contains "StartupWMClass=chrome-abc-Default")) {
        error make {msg: "ERROR: StartupWMClass missing correct value"}
    }

    let needs_rename = ($apps_dir | path join "needs_rename.desktop")
    if (not ($needs_rename | path exists)) {
        error make {msg: "ERROR: needs_rename.desktop was unexpectedly renamed"}
    }

    print "--- Test Case 2: With --rename ---"
    reset_fixtures $apps_dir
    chromium-pwa-wmclass-sync --apps-dir $apps_dir --rename -v
    
    print "Verifying results (with rename)..."
    
    let files = (ls $apps_dir | get name | path basename)
    print "Files in directory:"
    print $files

    # 1. Mismatched WMClass -> Should be renamed
    let p1 = ($apps_dir | path join "Mismatched WMClass.desktop")
    if (not ($p1 | path exists)) {
        error make {msg: $"ERROR: Mismatched WMClass.desktop not found. Found: ($files | str join ', ')"}
    }
    
    # 2. Correct App -> Should exist
    let p2 = ($apps_dir | path join "Correct App.desktop")
    if (not ($p2 | path exists)) {
        error make {msg: "ERROR: Correct App.desktop missing"}
    }
    
    # 3. New Name -> Should be renamed
    let p3 = ($apps_dir | path join "New Name.desktop")
    if (not ($p3 | path exists)) {
        error make {msg: "ERROR: New Name.desktop not found"}
    }
    
    # 4. Collision Source -> Should be renamed to numbered version
    let p4 = ($apps_dir | path join "Collision Target (1).desktop")
    if (not ($p4 | path exists)) {
        error make {msg: $"ERROR: Collision Target (1).desktop not found. Found: ($files | str join ', ')"}
    }
    
    # 5. Non PWA -> Should remain unchanged
    let p5 = ($apps_dir | path join "non_pwa.desktop")
    if (not ($p5 | path exists)) {
        error make {msg: "ERROR: non_pwa.desktop missing"}
    }

    print "Functional tests passed!"
}