echo "Executing Post Create Commands"
echo "Setting Yarn Version" \
&& corepack enable \
&& corepack install --global pnpm yarn@4.3.1 \
&& yarn set version stable
