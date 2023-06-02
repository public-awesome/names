for d in packages/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo publish
    cd ../..
  fi
done

cd contracts/sg721-name && cargo publish && cd ../../..

