# Define the path to Cargo.toml
$cargoTomlPath = "./Cargo.toml"

# Ensure Cargo.toml exists
if (-Not (Test-Path $cargoTomlPath)) {
    Write-Output "Cargo.toml not found at path: $cargoTomlPath"
    exit 1
}

# Read the Cargo.toml content into a variable
$cargoTomlContent = Get-Content -Path $cargoTomlPath -Raw

# Use a regular expression to find the version line
$matched = $cargoTomlContent -match 'version = "([^"]+)"'

if (-Not $matched) {
    Write-Output "Version line not found in Cargo.toml"
    exit 1
}

$versionLine = $matches[1]

# Split the version into major, minor, and patch
$versionParts = $versionLine.Split('.')
$major = $versionParts[0]
$minor = $versionParts[1]
$patch = [int]$versionParts[2]

# Increment the patch version
$patch += 1

# Construct the new version string
$newVersion = "$major.$minor.$patch"

# Replace the old version with the new version in the Cargo.toml content
$newCargoTomlContent = $cargoTomlContent -replace ('version = "' + [regex]::Escape($versionLine) + '"'), ('version = "' + $newVersion + '"')

# Write the new Cargo.toml content back to the file
Set-Content -Path $cargoTomlPath -Value $newCargoTomlContent

# Debug: Output the new version to verify it's correct
Write-Output "Updated version to $newVersion in Cargo.toml"

# add ALL files to git
git add .

# Commit the change with a message indicating the new version
git commit -m "bump to 🚀 $newVersion and release 🎉"

# Tag the commit as a release
git tag -a "v$newVersion" -m "Release v$newVersion"

# Push the commit and tag to your repository
git push && git push --tags
