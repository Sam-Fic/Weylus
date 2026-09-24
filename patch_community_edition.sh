git clone https://github.com/H-M-H/Weylus.git
cd Weylus
echo "CONTRIBUTORS merge=ours" >.gitattributes
git config --global merge.ours.driver true


git remote add lyonbot https://github.com/lyonbot/Weylus.git
git fetch lyonbot pr1
git merge lyonbot/pr1 -m "merged lyonbot"

git remote add electronstudio2 https://github.com/electronstudio2/Weylus.git
git fetch electronstudio2 build-fixes
git merge electronstudio2/build-fixes -m "merged build-fixes"

sed -i '/<<<<<<< HEAD/,/=======/d' .github/workflows/build.yml
sed -i '/>>>>>>> electronstudio2\/build-fixes/d' .github/workflows/build.yml

sed -i '/<<<<<<< HEAD/d' build.rs
sed -i '/=======/,/>>>>>>> electronstudio2\/build-fixes/d' build.rs
sed -i '/>>>>>>> electronstudio2\/build-fixes/d' build.rs

git add build.rs .github/workflows/build.yml
git commit -m 'resolve conflict'

git fetch electronstudio2 community-edition-patches
git merge electronstudio2/community-edition-patches -m "merged community-edition-patches"

git remote add wce https://github.com/electronstudio/WeylusCommunityEdition.git
###git push -u wce master --force
