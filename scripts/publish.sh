for d in packages/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo publish
    cd ../..
  fi
done

for d in contracts/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo publish
    cd ../..
  fi
done
