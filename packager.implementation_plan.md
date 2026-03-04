# 🔹 Implementation Plan — Packager `create template`

## 1. Objetivo

Crear la funcionalidad que genere un archivo `.wallman` empaquetando:

* `manifest.toml` → representación de `Config` en TOML.
* Carpeta `images/` → todos los archivos dentro de `self.path/images`, solo los que sean imágenes válidas.
* Comprimir en tar + zstd.

---

## 2. Estructura del flujo

```
Packager::pack(out_path)
 ├─ 1. Validar que self.path existe y es un directorio
 ├─ 2. Crear un archivo tar en memoria o en disco temporal
 │    ├─ Escribir manifest.toml
 │    └─ Recorrer self.path/images y añadir solo imágenes válidas
 ├─ 3. Comprimir tar con zstd
 └─ 4. Guardar archivo final en out_path
```

---

## 3. Dependencias de crates

1. `serde` + `toml` → serializar `Config` a TOML (`toml::to_vec`).
2. `tar` → generar archivo tar en memoria o temporal.
3. `zstd` → comprimir el tar a formato `.wallman`.
4. `image` crate → validar que los archivos sean imágenes válidas (`image::io::Reader::open(path)`).

Opcional: `walkdir` para recorrer imágenes recursivamente (si planeas soporte futuro).

---

## 4. Pasos detallados

### Paso 1: Validar paths

* Comprobar que `self.path` existe y es un directorio.
* Comprobar que `self.path/images` existe.
* Retornar error si no se cumple.

### Paso 2: Serializar configuración

* Usar `toml::to_vec(&self.config)` → obtiene `Vec<u8>` con manifest.
* Nombre del archivo: `manifest.toml` dentro del tar.

### Paso 3: Crear archivo tar en memoria

* Usar `tar::Builder` con un `std::io::Cursor<Vec<u8>>` o un archivo temporal.
* Añadir `manifest.toml` al tar:

  ```rust
  let mut header = tar::Header::new_gnu();
  header.set_size(manifest_bytes.len() as u64);
  header.set_mode(0o644);
  header.set_cksum();
  tar_builder.append_data(&mut header, "manifest.toml", &manifest_bytes[..])?;
  ```

### Paso 4: Añadir imágenes válidas

* Recorrer todos los archivos en `self.path/images`:

  ```rust
  for entry in std::fs::read_dir(&self.path.join("images"))? {
      let path = entry?.path();
      if path.is_file() && is_image(&path)? {
          tar_builder.append_path_with_name(&path, format!("images/{}", path.file_name().unwrap().to_string_lossy()))?;
      }
  }
  ```
* `is_image(path: &Path) -> Result<bool>`:

  * Abrir con `image::io::Reader::open(path)` y verificar si puede decodificarse.
  * Retornar `true` si es imagen, `false` si no.

### Paso 5: Comprimir tar con zstd

* Crear archivo de salida: `let out_file = File::create(out_path)?;`
* Usar `zstd::stream::write::Encoder::new(out_file, level)` para comprimir tar.
* Escribir todo el contenido del tar en el encoder.

### Paso 6: Finalizar

* Llamar `encoder.finish()?` para cerrar correctamente.
* Retornar `Ok(())` si todo salió bien.

---

## 5. Funciones auxiliares

```rust
fn is_image(path: &Path) -> std::io::Result<bool> {
    match image::io::Reader::open(path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
```

Opcionalmente, se puede añadir logging con `tracing` para informar qué archivos se omiten.

---

## 6. Consideraciones de seguridad y robustez

* Validar nombres de archivos para evitar paths relativos peligrosos (`../`) en el tar.
* Manejar errores parciales sin abortar todo el empaquetado (log de advertencia si alguna imagen no es válida).
* Soporte futuro para metadatos adicionales en manifest (`author`, `version`, etc.).

---

## 7. Roadmap de desarrollo

1. Implementar `pack()` siguiendo los pasos anteriores.
2. Crear tests unitarios:

   * Pack con imágenes válidas.
   * Pack con archivos no-imagen.
   * Pack con path de imágenes vacío.
3. Integrar logging con `tracing` para debug de empaquetado.
4. Validar compatibilidad con `flatpak` paths (~/.local/share).
5. Optimizar memoria si se trabaja con muchos archivos grandes (usar tar streaming directamente al encoder).
