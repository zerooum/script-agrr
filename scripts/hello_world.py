"""Script de teste do agrr — sem dependências externas."""

import sys
sys.path.insert(0, "sdk/python")

from agrr_sdk import AgrrScript


class HelloWorld(AgrrScript):
    name = "Hello World"
    description = "Exibe uma saudação — script de teste"
    group = "exemplos"
    version = "1.0.0"
    runtime = {"language": "python", "min_version": "3.8"}

    args = [
        {"name": "nome", "prompt": "Qual é o seu nome?"},
        {
            "name": "idioma",
            "prompt": "Idioma?",
            "options": ["pt", "en", "es"],
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        nome = args.get("nome") or "Mundo"
        idioma = args.get("idioma", "pt")

        saudacoes = {"pt": "Olá", "en": "Hello", "es": "Hola"}
        saudacao = saudacoes.get(idioma, "Olá")

        print(f"{saudacao}, {nome}!")
        print("Script de teste executado com sucesso. ✓")


if __name__ == "__main__":
    HelloWorld.main()
