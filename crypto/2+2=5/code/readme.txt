# Author: @gio-d3141
# Dockerfile should build fine (will take a while) but the setup is kind of cursed, DM me if necessary

git clone git@github.com:a16z/jolt.git
cd jolt

# just the latest commit at time of writing
git checkout 0cc7aa31981ff8503fe256706d2aa9c320abd1cd
git apply ../diff.patch
