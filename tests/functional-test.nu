#!/usr/bin/env nu

# Nushell functional test for chromium-pwa-wmclass-sync

def reset_fixtures [apps_dir: string, fixture_dir: string] {
    # Delete everything in the directory
    if (ls $apps_dir | length) > 0 {
        ls $apps_dir | each {|f| rm -rf $f.name }
    }

    # Copy fixtures
    ls $fixture_dir | each {|f| cp $f.name $apps_dir }

    # Ensure writable
    ls $apps_dir | each {|f| chmod +w $f.name }
}

def main [] {
    let apps_dir = (mktemp -d)

    print "--- Test Case 1: First run ---"
    reset_fixtures $apps_dir "tests/fixtures"
    chromium-pwa-wmclass-sync --apps-dir $apps_dir -v

    print "Verifying results (first run)..."

    let files = (ls $apps_dir | get name | path basename)
    print $"Files in directory: ($files | str join ', ')"

    # Check original is neutered
    let original = (open ($apps_dir | path join "chrome-abc-Default.desktop") --raw)
    if (not ($original | str contains "NoDisplay=true")) {
        error make {msg: "ERROR: Original not neutered (missing NoDisplay=true)"}
    }
    if ($original | str contains "StartupWMClass=") {
        error make {msg: "ERROR: Original still has StartupWMClass"}
    }
    if (not ($original | str contains "X-ChromiumPwaSyncManaged=true")) {
        error make {msg: "ERROR: Original missing managed marker"}
    }

    # Check companion exists with correct content
    let companion_path = ($apps_dir | path join "Test App.desktop")
    if (not ($companion_path | path exists)) {
        error make {msg: $"ERROR: Test App.desktop companion not found. Found: ($files | str join ', ')"}
    }
    let companion = (open $companion_path --raw)
    if (not ($companion | str contains "StartupWMClass=chrome-abc-Default")) {
        error make {msg: "ERROR: Companion missing correct StartupWMClass"}
    }
    if (not ($companion | str contains "X-ChromiumPwaSyncSource=chrome-abc-Default.desktop")) {
        error make {msg: "ERROR: Companion missing sync source marker"}
    }
    if ($companion | str contains "NoDisplay") {
        error make {msg: "ERROR: Companion should not have NoDisplay"}
    }

    # Check collision handling (chrome-jkl has Name=Collision Target, collides with non-PWA)
    let collision_companion = ($apps_dir | path join "Collision Target (1).desktop")
    if (not ($collision_companion | path exists)) {
        error make {msg: $"ERROR: Collision Target (1).desktop not found. Found: ($files | str join ', ')"}
    }
    # chrome-jkl uses an absolute Icon= path; WM class must come from the filename
    let collision = (open $collision_companion --raw)
    if (not ($collision | str contains "StartupWMClass=chrome-jkl-Default")) {
        error make {msg: "ERROR: Collision companion has wrong StartupWMClass (absolute Icon path)"}
    }

    # Check non-PWA unchanged
    if (not (($apps_dir | path join "non_pwa.desktop") | path exists)) {
        error make {msg: "ERROR: non_pwa.desktop missing"}
    }
    if (not (($apps_dir | path join "Collision Target.desktop") | path exists)) {
        error make {msg: "ERROR: Collision Target.desktop (non-PWA) missing"}
    }

    # Should be 6 files: 2 originals (neutered) + 2 companions + 2 non-PWAs
    let file_count = (ls $apps_dir | length)
    if ($file_count != 6) {
        error make {msg: $"ERROR: Expected 6 files, got ($file_count): ($files | str join ', ')"}
    }

    print "--- Test Case 2: Idempotent re-run ---"
    let before_original = (open ($apps_dir | path join "chrome-abc-Default.desktop") --raw)
    let before_companion = (open ($apps_dir | path join "Test App.desktop") --raw)
    chromium-pwa-wmclass-sync --apps-dir $apps_dir -v
    let after_original = (open ($apps_dir | path join "chrome-abc-Default.desktop") --raw)
    let after_companion = (open ($apps_dir | path join "Test App.desktop") --raw)
    if ($before_original != $after_original) {
        error make {msg: "ERROR: Original changed on idempotent run"}
    }
    if ($before_companion != $after_companion) {
        error make {msg: "ERROR: Companion changed on idempotent run"}
    }
    let file_count = (ls $apps_dir | length)
    if ($file_count != 6) {
        error make {msg: $"ERROR: File count changed on idempotent run: ($file_count)"}
    }

    print "--- Test Case 3: Chrome regeneration ---"
    # Simulate Chrome overwriting the neutered original with fresh content
    cp tests/fixtures/chrome-abc-Default.desktop ($apps_dir | path join "chrome-abc-Default.desktop")
    chromium-pwa-wmclass-sync --apps-dir $apps_dir -v

    let original = (open ($apps_dir | path join "chrome-abc-Default.desktop") --raw)
    if (not ($original | str contains "X-ChromiumPwaSyncManaged=true")) {
        error make {msg: "ERROR: Re-neutering failed after Chrome regeneration"}
    }
    if ($original | str contains "StartupWMClass=") {
        error make {msg: "ERROR: Original still has StartupWMClass after re-neutering"}
    }
    let companion = (open ($apps_dir | path join "Test App.desktop") --raw)
    if (not ($companion | str contains "StartupWMClass=chrome-abc-Default")) {
        error make {msg: "ERROR: Companion not recreated after Chrome regeneration"}
    }

    print "--- Test Case 4: Orphan cleanup ---"
    # Simulate PWA uninstall: Chrome deletes the original
    rm ($apps_dir | path join "chrome-abc-Default.desktop")
    chromium-pwa-wmclass-sync --apps-dir $apps_dir -v

    if (($apps_dir | path join "Test App.desktop") | path exists) {
        error make {msg: "ERROR: Orphaned companion not deleted"}
    }
    # Other files should still be present
    if (not (($apps_dir | path join "chrome-jkl-Default.desktop") | path exists)) {
        error make {msg: "ERROR: chrome-jkl-Default.desktop missing after orphan cleanup"}
    }
    if (not (($apps_dir | path join "Collision Target (1).desktop") | path exists)) {
        error make {msg: "ERROR: Collision Target (1).desktop missing after orphan cleanup"}
    }

    print "Functional tests passed!"
}
