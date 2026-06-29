# Utilizar una imagen oficial de Rust (basada en Debian)
FROM rust:slim-bookworm

# Instalar dependencias del sistema, Python y herramientas requeridas para OpenSSL
RUN apt-get update && apt-get install -y python3 python3-pip python3-venv pkg-config libssl-dev

# Crear un entorno virtual de Python y agregarlo al PATH
ENV VIRTUAL_ENV=/opt/venv
RUN python3 -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

# Instalar spaCy y descargar el modelo lingüístico en español
RUN pip install --upgrade pip setuptools wheel && \
    pip install spacy click && \
    python -m spacy download es_core_news_sm

# Configurar el directorio de trabajo dentro del contenedor
WORKDIR /usr/src/app

# Copiar todo el código fuente y archivos del alumno al contenedor
COPY . .

# Compilar el proyecto en modo release
RUN cargo build --release

# Comando por defecto al ejecutar el contenedor
CMD ["cargo", "run", "--release"]