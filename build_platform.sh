#!/usr/bin/env bash

PLATFORM=$1
TEMPNAME=cont1_${RANDOM}

mkdir -p platforms/${PLATFORM}/target
rm -rf platforms/${PLATFORM}/target

# Do the build
docker build -t git-poly:${PLATFORM} -f platforms/${PLATFORM}/Dockerfile .
docker create --name ${TEMPNAME} git-poly:${PLATFORM}
docker cp ${TEMPNAME}:/target platforms/${PLATFORM}/target
docker rm ${TEMPNAME}
