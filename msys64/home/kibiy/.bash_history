./Configure mingw64 no-shared no-dso
cd /d/city_chain_project/local_openssl_src
./Configure mingw64 no-shared no-dso
cd /d/city_chain_project/msys64/home/openssl-3.3.1
./Configure mingw64 no-shared no-dso
make
pacman -S perl
perl -v
perl Configure mingw64 no-shared no-dso
pacman -S perl
perl -v
which perl
cd /d/city_chain_project/msys64/home/openssl-3.3.1
perl Configure mingw64 no-shared no-dso
echo $PATH
