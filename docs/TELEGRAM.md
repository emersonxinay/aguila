# 🤖 Documentación Oficial: Módulo Telegram para Águila

El módulo `telegram` permite construir bots interactivos de Telegram de forma nativa en español utilizando el lenguaje **Águila**.

---

## 📦 Importación

```aguila
importar env
importar telegram
```

---

## 🚀 Inicialización

Para crear una instancia del bot:

```aguila
mi_bot = nuevo telegram.MotorBot()
```

---

## 🛠️ Métodos Disponibles

### 1. `al_recibir(comando, funcion_manejadora)`
Registra un manejador para un comando o texto específico (ej. `"iniciar"`, `"/start"`, `"ayuda"`).

```aguila
funcion bienvenida(texto, chat_id, bot) {
    bot.responder(chat_id, "¡Hola! Bienvenido a mi bot de Águila.")
}

mi_bot.al_recibir("iniciar", bienvenida)
mi_bot.al_recibir("/start", bienvenida)
```

#### 🌟 Manejo de Comandos No Reconocidos (Wildcard `*`)
Puedes registrar un manejador especial usando `"*"` para capturar cualquier mensaje que no coincida con los comandos registrados:

```aguila
funcion comando_no_reconocido(texto, chat_id, bot) {
    bot.responder(chat_id, "⚠️ El comando '" + texto + "' no es válido. Escribe 'iniciar' para ver las opciones.")
}

mi_bot.al_recibir("*", comando_no_reconocido)
```

---

### 2. `responder(chat_id, texto)`
Envía un mensaje de texto simple al usuario.

```aguila
bot.responder(chat_id, "Tu mensaje fue recibido con éxito.")
```

---

### 3. `responder_con_botones_en_linea(chat_id, texto, botones)`
Envía un mensaje con botones interactivos en línea (Inline Keyboards) que admiten URLs o datos de callback.

```aguila
botones = [
    [
        {"text": "📌 Opción 1", "callback_data": "opcion_1"},
        {"text": "🌐 Sitio Web", "url": "https://ejemplo.com"}
    ]
]

bot.responder_con_botones_en_linea(chat_id, "Selecciona una opción:", botones)
```

---

### 4. `responder_con_teclado(chat_id, texto, botones)`
Envía un mensaje con botones de teclado de respuesta (Reply Keyboard).

```aguila
botones = [
    ["Opción A", "Opción B"],
    ["Ayuda"]
]

bot.responder_con_teclado(chat_id, "Elige una opción:", botones)
```

---

### 5. `arrancar(token)`
Inicia el ciclo continuo de recepción de mensajes (Long Polling).

```aguila
gestor_env = nuevo env.GestorEnv()
gestor_env.configurar(".env")

token = gestor_env.obtener("TELEGRAM_TOKEN")
mi_bot.arrancar(token)
```

---

## 📝 Ejemplo Genérico Completo

```aguila
importar env
importar telegram

funcion inicio(texto, chat_id, bot) {
    botones = [
        [{"text": "Ver información", "callback_data": "info"}]
    ]
    bot.responder_con_botones_en_linea(chat_id, "Bienvenido al Bot de Ejemplo", botones)
}

funcion mostrar_info(texto, chat_id, bot) {
    bot.responder(chat_id, "Esta es una respuesta genérica del sistema.")
}

funcion fallback(texto, chat_id, bot) {
    bot.responder(chat_id, "No reconocí el mensaje '" + texto + "'. Escribe 'iniciar'.")
}

bot = nuevo telegram.MotorBot()
bot.al_recibir("iniciar", inicio)
bot.al_recibir("info", mostrar_info)
bot.al_recibir("*", fallback)

env_manager = nuevo env.GestorEnv()
env_manager.configurar(".env")
bot.arrancar(env_manager.obtener("TELEGRAM_TOKEN"))
```
