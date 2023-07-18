# example/golang

## Usage

First we want to automatically generate the YAML defining our folder structure.
You may also opt to do this manually if you prefer.

```bash
folders pull example/golang/*
```

Output YAML:
```yaml
- README.md
- cmd/:
  - main/
- internal/
- pkg/:
  - test/:
    - test.go
  - pkg.go
- vendor/
```

Next we want to generate the folder structure from the YAML.

```bash
folders push example/golang-result
```