# Getting started

## Download

|Platform|    Version    |
|--------|---------------|
|Linux   | Kernel 4.9.x  |

## 101

### Cloning

```bash
echo https://github.com/openssl/openssl.git >> clone.txt
echo https://github.com/libjpeg-turbo/libjpeg-turbo.git >> clone.txt
cat clone.txt | git poly clone
rm clone.txt
```
