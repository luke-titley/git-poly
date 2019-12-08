# Remove the old build
mkdir -p centos7/target
rm -rf centos7/target

# Do the build
docker build -t git-poly:centos7 -f centos7/Dockerfile .
docker create --name cont1 git-poly:centos7
docker cp cont1:/target centos7/target
docker rm cont1
