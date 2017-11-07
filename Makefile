publish:
	cat README-header.md > README.md
	cargo readme --no-title >> README.md
	rustup run stable cargo publish

.PHONY: publish
