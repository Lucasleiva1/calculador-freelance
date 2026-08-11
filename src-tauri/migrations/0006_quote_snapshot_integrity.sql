-- Los snapshots son evidencia histórica. La eliminación definitiva continúa
-- siendo posible al eliminar su cotización archivada, pero una fila existente
-- no puede ser alterada accidentalmente por código presente o futuro.
CREATE TRIGGER IF NOT EXISTS quote_snapshots_are_immutable
BEFORE UPDATE ON quote_snapshots
BEGIN
  SELECT RAISE(ABORT, 'Los snapshots históricos son inmutables. Creá una nueva revisión.');
END;
