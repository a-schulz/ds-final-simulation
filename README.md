# Simulation eines SMD Fertigungsprozesses

## Situation:
Beim Referenzprodukt TELIX1 existiert folgender Arbeitsablauf:

Bestückung der Leiterplatte mit SMD-Bauteilen auf einem SMD-Automaten (1 min konstant)
Löten der Leiterplatten im Lotbad (6 Stück pro Durchlauf, 10 Durchläufe / h ) 6 min pro Durchlauf
Montage (Anbringen von Buchsen und Einbau in Gehäuse) (4 min +1/-1 min gleichverteilt)
Qualitätskontrolle (elektronischer Funktionstest und optische Sichtkontrolle) und Kom-
plettierung mit Zubehör und Dokumentation, Verpackung und Versand (normalverteilt mit
Mittelwert = 6 min und Standardabweichung = 4 min, aber mindestens 3 Minuten!)

Folgende Ressourcen sind vorhanden (s. Layout):
- zentrales Bauelemente- und Teilelager mit ausreichend Teilen für die Beschickung des SMD-Automaten
- ein SMD-Automat
- ein Schwalllotbad,
- 2 Handarbeitsplätze für die Montage
- 2 Testarbeitsplätze für Ausgangskontrolle und Versand
- Vor den Fertigungsbereichen Lotbad, Handarbeit und Test ist jeweils ein Pufferlager mit einer Kapazität von je 15 Teilen angeordnet (P1,P2,P3).
