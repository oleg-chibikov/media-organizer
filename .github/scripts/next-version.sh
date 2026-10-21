#!/bin/sh
# Prints the next release version after the highest vX.Y.Z tag.
#   next-version.sh patch    v0.2.0 -> 0.2.1
#   next-version.sh minor    v0.2.0 -> 0.3.0
#   next-version.sh major    v0.2.0 -> 1.0.0
set -eu

bump=${1:-patch}
latest=$(git tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-version:refname | head -n 1)
latest=${latest:-v0.0.0}

echo "${latest#v}" | awk -F. -v bump="$bump" '{
  if (bump == "major") print $1 + 1 ".0.0"
  else if (bump == "minor") print $1 "." $2 + 1 ".0"
  else if (bump == "patch") print $1 "." $2 "." $3 + 1
  else { print "Unknown bump: " bump > "/dev/stderr"; exit 1 }
}'
