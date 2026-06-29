"""
ner_detector.py

Recibe un conjunto de textos y devuelve las entidades nombradas
encontradas en formato JSON .

Protocolo:
  - in:  JSON donde las claves son identificadores (ej. nombre de archivo)
            y los valores son el texto completo de ese documento.
  - out: JSON con la misma estructura de claves, pero donde el valor
            es un objeto conteniendo la lista de entidades encontradas.
  - err: mensajes de error.
  - exit 0: ok  |  exit 1: error.

Ejemplo de entrada (stdin):
  {
    "doc1.txt": "Elon Musk fundó Tesla.",
    "doc2.txt": "YPF es una empresa de Argentina."
  }

Ejemplo de salida (stdout):
  {
    "doc1.txt": { "entities": [ {"text": "Elon Musk", "label": "PER"}, {"text": "Tesla", "label": "ORG"} ] },
    "doc2.txt": { "entities": [ {"text": "YPF", "label": "ORG"}, {"text": "Argentina", "label": "LOC"} ] }
  }

Etiquetas normalizadas:
  PER  → personas
  ORG  → organizaciones
  LOC  → lugares / ubicaciones
  MISC → otras entidades nombradas
"""


import sys
import json
import argparse

# ── Mapeo de etiquetas spaCy → etiquetas normalizadas ────────────────────────
LABEL_MAP = {
    "PER":   "PER",
    "PERS":  "PER",
    "ORG":   "ORG",
    "LOC":   "LOC",
    "LUGAR": "LOC",
    "MISC":  "MISC",
    "PERSON":   "PER",
    "GPE":      "LOC",
    "FACILITY": "LOC",
    "NORP":     "MISC",
    "PRODUCT":  "MISC",
    "EVENT":    "MISC",
    "WORK_OF_ART": "MISC",
    "LAW":      "MISC",
    "LANGUAGE": "MISC",
}

DEFAULT_MODEL = "es_core_news_sm"

def load_model(model_name: str):
    try:
        import spacy
        return spacy.load(model_name)
    except OSError:
        print(
            f"[ner_detector] Modelo '{model_name}' no encontrado.\n"
            f"  Instalalo con:  python3 -m spacy download {model_name}",
            file=sys.stderr
        )
        sys.exit(1)
    except ImportError:
        print(
            "[ner_detector] spaCy no está instalado.\n"
            f"  Instalalo con:  pip install spacy",
            file=sys.stderr
        )
        sys.exit(1)

def extract_entities(nlp, text: str) -> list[dict]:
    doc = nlp(text)
    seen   = set()
    result = []

    for ent in doc.ents:
        normalized_label = LABEL_MAP.get(ent.label_, "MISC")
        key = (ent.text.strip(), normalized_label)

        if key in seen:
            continue
        seen.add(key)

        if ent.text.strip():
            result.append({
                "text":  ent.text.strip(),
                "label": normalized_label
            })

    return result

def main():
    parser = argparse.ArgumentParser(description="NER detector via spaCy (Batch mode)")
    parser.add_argument(
        "--model", default=DEFAULT_MODEL,
        help=f"Nombre del modelo spaCy a usar (default: {DEFAULT_MODEL})"
    )
    args = parser.parse_args()

    # Leer toda la entrada estándar
    input_data = sys.stdin.read()
    if not input_data.strip():
        print(json.dumps({}))
        return

    # Parsear el JSON de entrada
    try:
        docs = json.loads(input_data)
    except json.JSONDecodeError:
        print("[ner_detector] Error: La entrada no es un JSON válido.", file=sys.stderr)
        sys.exit(1)

    # Cargar el modelo una sola vez
    nlp = load_model(args.model)
    
    # Procesar todos los documentos
    results = {}
    for doc_id, text in docs.items():
        if not text.strip():
            results[doc_id] = {"entities": []}
        else:
            results[doc_id] = {"entities": extract_entities(nlp, text)}

    # Imprimir el resultado final
    print(json.dumps(results, ensure_ascii=False))

if __name__ == "__main__":
    main()