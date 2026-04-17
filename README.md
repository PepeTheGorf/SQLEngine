# Simple SQL Parser

Minimal SQL parser sa podrškom za osnovne DDL, DML i upitne (QL) operacije. Projekat omogućava rad sa jednostavnim tabelama kroz kreiranje strukture, unos podataka i izvršavanje SELECT upita sa osnovnim izrazima i uslovima.

---

## Podržane funkcionalnosti

### DDL (Data Definition Language)

* CREATE TABLE

### DML (Data Manipulation Language)

* INSERT INTO

### QL (Query Language)

* SELECT

  * izbor kolona
  * aliasi (AS)
  * aritmetički izrazi
  * WHERE uslovi

---

## Primer upotrebe

```sql
-- Kreiranje tabele
CREATE TABLE Radnik (
    ID Integer,
    Naziv VARCHAR(30),
    Plata Integer,
    Bonus Integer
);

-- Ubacivanje podataka
INSERT INTO Radnik VALUES 
    (1, 'Pera', 100, 100),
    (2, 'Mika', 200, 150),
    (3, 'Test3', 500, 250);

-- Osnovni SELECT
SELECT * FROM Radnik;

-- SELECT sa uslovom
SELECT * FROM Radnik WHERE Plata > 500;

-- Projekcija kolona
SELECT ID, Naziv FROM Radnik;

-- Izrazi i aliasi
SELECT Plata + Bonus AS "CelaPlata" FROM Radnik;

-- Kompleksniji primer
SELECT 
    ID AS "Identifikator",
    Naziv AS "Naziv",
    (Plata + 500) * 2 AS "Plata"
FROM Radnik
WHERE Plata > 300;
```

### Primer ispisa

```text
+------------------+------------------+------------------+
| Identifikator    | Naziv            | Plata            |
+------------------+------------------+------------------+
| 3                | Test3            | 2000             |
+------------------+------------------+------------------+
```
