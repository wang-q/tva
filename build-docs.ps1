# build-docs.ps1

$srcReadme = "README.md"
$destReadme = "docs/README.md"

Write-Host "Syncing $srcReadme to $destReadme..."

# Read the content of the source README
$content = Get-Content -Path $srcReadme -Raw

# Replace links: "docs/" -> "" (because the file is moving into docs/)
# Example: [Design Documentation](docs/design.md) -> [Design Documentation](design.md)
$content = $content -replace '\(docs/', '('

# Write to the destination
Set-Content -Path $destReadme -Value $content -Encoding UTF8

Write-Host "Sync complete."
Write-Host "Building mdBook..."

# Run mdbook build
mdbook build

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful! Output is in 'book/' directory."
} else {
    Write-Host "Build failed."
    exit $LASTEXITCODE
}
