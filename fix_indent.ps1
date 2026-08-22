$files = @("vnikey-wayland/src/main.rs", "vnikey-x11/src/main.rs")
foreach ($file in $files) {
    $lines = Get-Content $file
    for ($i = 0; $i -lt $lines.Length; $i++) {
        if ($lines[$i] -match '^( *)if let Some\(tray\) = ') {
            $indent = $matches[1]
            if ($i + 1 -lt $lines.Length -and $lines[$i+1] -match 'tray\.update') {
                $lines[$i+1] = "$indent    tray.update(|_| {});"
            }
            if ($i + 2 -lt $lines.Length -and $lines[$i+2] -match '\}') {
                $lines[$i+2] = "$indent}"
            }
        }
    }
    Set-Content -Path $file -Value $lines
}
