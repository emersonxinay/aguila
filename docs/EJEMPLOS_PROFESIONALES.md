#  Ejemplos Profesionales con Águila

Esta colección de ejemplos demuestra la potencia y flexibilidad del lenguaje Águila para casos de uso reales y algoritmos complejos.

---

## 1. Algoritmo de Dijkstra (Caminos Cortos)
Implementación clásica usando Diccionarios y lógica imperativa.

```aguila
usar "mate" # Para infinito (mate.inf)

clase Grafo {
    funcion init() {
        yo.nodos = {}
    }

    funcion agregar_eje(origen, destino, peso) {
        si no yo.nodos.contiene(origen) { yo.nodos[origen] = {} }
        si no yo.nodos.contiene(destino) { yo.nodos[destino] = {} }
        yo.nodos[origen][destino] = peso
        yo.nodos[destino][origen] = peso # Grafo no dirigido
    }

    funcion dijkstra(inicio) {
        let distancias = {}
        let visitados = []
        let nodos = yo.nodos.claves()

        # Inicializar distancias
        para nodo en nodos {
            distancias[nodo] = 1000000 # Infinito simulado
        }
        distancias[inicio] = 0

        mientras visitados.longitud() < nodos.longitud() {
            # Encontrar nodo con menor distancia no visitado
            let min_nodo = nulo
            let min_dist = 1000000
            
            para nodo en nodos {
                si (not visitados.contiene(nodo)) y (distancias[nodo] < min_dist) {
                    min_dist = distancias[nodo]
                    min_nodo = nodo
                }
            }

            si min_nodo == nulo { romper } # No hay más alcanzables
            visitados.agregar(min_nodo)

            # Relajar aristas
            let vecinos = yo.nodos[min_nodo]
            para vecino en vecinos.claves() {
                let peso = vecinos[vecino]
                let nueva_dist = distancias[min_nodo] + peso
                si nueva_dist < distancias[vecino] {
                    distancias[vecino] = nueva_dist
                }
            }
        }
        retornar distancias
    }
}

let g = nuevo Grafo()
g.agregar_eje("A", "B", 4)
g.agregar_eje("A", "C", 2)
g.agregar_eje("B", "C", 1)
g.agregar_eje("B", "D", 5)
g.agregar_eje("C", "D", 8)

imprimir("Distancias desde A: " + g.dijkstra("A"))
```

---

## 2. API REST con Nido (Clean Architecture)
Ejemplo de cómo estructurar una API profesionalmente.

```aguila
usar "http"
usar "json"
usar "db"

# --- Modelo ---
clase Usuario {
    funcion guardar(db_con) {
        let sql = a"INSERT INTO usuarios (nombre, email) VALUES ('{yo.nombre}', '{yo.email}')"
        db_con.ejecutar(sql)
    }
}

# --- Controlador ---
clase UsuarioControlador {
    funcion crear(req) {
        let datos = json.parsear(req.cuerpo)
        
        # Validación
        si no datos.contiene("email") {
            retornar { "estado": 400, "cuerpo": "Email requerido" }
        }

        let usuario = nuevo Usuario()
        usuario.nombre = datos["nombre"]
        usuario.email = datos["email"]
        
        intentar {
            usuario.guardar(req.db)
            retornar { "estado": 201, "cuerpo": "Usuario creado" }
        } capturar error {
            retornar { "estado": 500, "cuerpo": "Error: " + error }
        }
    }
}

# --- Router & App ---
let servidor = http.servidor(8080)
let db_pool = db.conectar("sqlite://usuarios.db")

servidor.middleware(funcion(req, next) {
    req.db = db_pool # Inyección de dependencia
    retornar next(req)
})

let control = nuevo UsuarioControlador()
servidor.ruta("POST", "/usuarios", control.crear)

imprimir("Servidor corriendo en puerto 8080...")
servidor.iniciar()
```

---

## 3. Concurrencia con Hilos (Productor/Consumidor)
Uso de multithreading para procesamiento paralelo.

```aguila
usar "thread"
usar "tiempo"

let cola = []
let bloqueo = thread.mutex()
let cond = thread.condvar()

funcion productor() {
    para i = 1 hasta 6 {
        bloqueo.adquirir()
        cola.agregar(i)
        imprimir(a"Producido: {i}")
        bloqueo.liberar()
        cond.notificar() # Avisar al consumidor
        tiempo.dormir(0.5)
    }
}

funcion consumidor() {
    mientras verdadero {
        bloqueo.adquirir()
        
        mientras cola.longitud() == 0 {
            cond.esperar(bloqueo) # Esperar señal
        }
        
        let item = cola.eliminar(0)
        imprimir(a"Consumido: {item}")
        bloqueo.liberar()
        
        si item == 5 { romper } # Terminar
    }
}

let t1 = thread.crear(productor)
let t2 = thread.crear(consumidor)

t1.unir()
t2.unir()
imprimir("Proceso terminado.")
```
