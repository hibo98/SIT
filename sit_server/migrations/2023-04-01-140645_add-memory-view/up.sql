CREATE VIEW memory AS SELECT client_id, SUM(capacity) AS capacity, COUNT(capacity) AS sticks FROM memory_stick GROUP BY client_id;