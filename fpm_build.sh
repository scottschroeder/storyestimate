#!/bin/bash

set -e

DEB_VERSION="${1}"

if [[ -z "${DEB_VERSION}" ]]; then
    echo "ERROR: Missing DEB_VERSION arg"
    exit 1
fi

NAME="story-estimates-webapp"
MAINTAINER="scottschroeder@sent.com"
OWNER="root"
GROUP="root"
DESCRIPTION="The core webapp for story estimates project"
ARCHITECTURE="all"
BUILD_DIR="./build/"

working_dir="$(/usr/bin/dirname "$(readlink -f $0)")"

cd "$working_dir"
mkdir -pv "$BUILD_DIR"

fpm --verbose --force -s dir -t deb -n "${NAME}" -v "${DEB_VERSION}" --deb-user "${OWNER}" --deb-group "${GROUP}" \
    --description "${DESCRIPTION}" --maintainer "${MAINTAINER}" --deb-compression xz \
    --architecture "${ARCHITECTURE}" \
    --package ${BUILD_DIR} \
    -s dir target/release/estimate=/opt/storyestimates/bin/estimate \
    templates=/opt/storyestimates/ \
    vendor/swagger-ui=/opt/storyestimates/ \
    config/nginx/storyestimates.conf=/opt/storyestimates/config/nginx/ \
    config/Rocket.toml=/opt/storyestimates/config/ \
