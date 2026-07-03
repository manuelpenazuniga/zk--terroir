# Verificación pública ZK-Terroir (QR / `lot_status`)

Verificador de **solo lectura**: dado un `lot_commit` (hex de 32 bytes), comprueba
on-chain en **Testnet** que ese lote de café fue **certificado y su premium pagado**,
sin revelar la cadena de suministro. No mueve fondos, no firma, no muta estado
(usa `stellar contract invoke --send=no`, pura simulación).

El contract id y la red se leen de [`../deployments/testnet.json`](../deployments/testnet.json)
(`addresses.terroir_contract`) — **nada hardcodeado**: si ese archivo cambia, el
verificador sigue el valor de ahí.

## Requisitos

- [`stellar`](https://developers.stellar.org/docs/tools/cli) CLI (ya presente en esta máquina).
- `jq` (para leer el JSON de despliegue).
- *(opcional)* `qrencode` para materializar el QR como PNG.

No hace falta instalar toolchains nuevos ni claves con permisos de escritura.

## Uso

```bash
# Identidad solo-consulta para la simulación (--send=no no firma ni paga fee).
# Por defecto usa la identidad local 'terroir'; si no la tienes, pasa la tuya:
export STELLAR_SOURCE=<tu_identidad_stellar>

./verify.sh <lot_commit_hex_64>
```

**Códigos de salida:** `0` = certificado (encontrado) · `1` = no encontrado /
no reclamado (respuesta válida) · `2` = uso incorrecto · `3` = error de red/CLI.

### Ejemplo con el lote del E2E real (existe en Testnet)

```bash
./verify.sh 2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563
```

Salida esperada:

```
contrato  : CBHAL7G57DPXXAR4BAZIXMSU3LUGPW4FPNYKPSRKZP3I5C4LP5STCM5W  (testnet)
lot_commit: 2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563
✓ Lote certificado — premium pagado el ts=1783036704
```

Con un `lot_commit` inventado:

```bash
./verify.sh 00000000000000000000000000000000000000000000000000000000deadbeef
# → ✗ No encontrado / no reclamado
```

## QR

`gen_qr.sh` codifica un payload autoexplicativo que apunta a la consulta del lote:

```
zkterroir:verify?lot_commit=<hex>&contract=<CID>&network=testnet
```

```bash
./gen_qr.sh 2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563
```

Si `qrencode` está instalado escribe un PNG; si no, imprime el payload e
instrucciones para generarlo. Un consumidor/juez escanea el QR y corre
`verify.sh <lot_commit>` (o una página que llame a `lot_status`) para confirmar
la certificación y el pago del premium.

## Qué prueba (y qué no)

- **Prueba:** que el contrato `terroir` registró ese `lot_commit` con un timestamp
  de claim ⇒ hubo una prueba Groth16 válida que pasó root+floor+nullifier+payout y
  pagó el premium (ver [`../README.md`](../README.md)).
- **No revela** la cadena de suministro ni ningún secreto: `lot_status` solo expone
  el timestamp del claim.
