
default:
	@echo "redeploy | build | podman | install"

redeploy: build podman install

build:
	cd subparsvc/subparweb && cargo build --release

podman: build
	-podman image rm localhost/deploy_api
	-podman container stop subparsvc
	-podman container rm subparsvc
	podman compose -f deploy/compose.yaml up -d

install:
	rm -rf /var/www/subpar
	mkdir -p /var/www/subpar
	cp -r site/* /var/www/subpar
	chown -R caddy /var/www/subpar
	chmod -R -w /var/www/subpar
	mkdir -p /var/log/caddy
	chown -R caddy /var/log/caddy
	cp deploy/Caddyfile /etc/caddy/Caddyfile
	systemctl start caddy
	systemctl status caddy | more

