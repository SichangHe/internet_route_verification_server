
-- Create Trigger to update rpsl_obj Table:
-- CREATE OR REPLACE FUNCTION trg_update_rpsl_obj()
-- RETURNS TRIGGER AS $$
-- BEGIN
--   update rpsl_obj_mnt_by set rpsl_obj_name=NEW.rpsl_obj_name where rpsl_obj_name=OLD.rpsl_obj_name;
--   update aut_num set rpsl_obj_name=NEW.rpsl_obj_name where rpsl_obj_name=OLD.rpsl_obj_name;
--   update peering_set set peering_set_name=NEW.rpsl_obj_name where peering_set_name=OLD.rpsl_obj_name;
--   update filter_set set filter_set_name=NEW.rpsl_obj_name where filter_set_name=OLD.rpsl_obj_name;
--   update route_set set rpsl_obj_name=NEW.rpsl_obj_name where rpsl_obj_name=OLD.rpsl_obj_name;
--   update as_set set as_set_name=NEW.rpsl_obj_name where as_set_name=OLD.rpsl_obj_name;
--   update mbrs_by_ref set rpsl_obj_name=NEW.rpsl_obj_name where rpsl_obj_name=OLD.rpsl_obj_name;
--   update route_set set route_set_name=NEW.rpsl_obj_name where route_set_name=OLD.rpsl_obj_name;
--   RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- CREATE TRIGGER trg_update_rpsl_obj
-- AFTER UPDATE ON rpsl_obj
-- FOR EACH ROW
-- EXECUTE FUNCTION trg_update_rpsl_obj();
------------------------------------------------------------------------------------------------------

-- Create Trigger to delete rpsl_obj Table:
-- CREATE OR REPLACE FUNCTION trg_delete_rpsl_obj()
-- RETURNS TRIGGER AS $$
-- BEGIN
--   delete from rpsl_obj_mnt_by  where rpsl_obj_name=OLD.rpsl_obj_name;
--   delete from aut_num where rpsl_obj_name=OLD.rpsl_obj_name;
--   delete from peering_set where peering_set_name=OLD.rpsl_obj_name;
--   delete from filter_set where filter_set_name=OLD.rpsl_obj_name;
--   delete from route_set where rpsl_obj_name=OLD.rpsl_obj_name;
--   delete from as_set where as_set_name=OLD.rpsl_obj_name;
--   delete from mbrs_by_ref where rpsl_obj_name=OLD.rpsl_obj_name;
--   delete from route_set where route_set_name=OLD.rpsl_obj_name;
--   RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- CREATE TRIGGER trg_delete_rpsl_obj
-- AFTER DELETE ON rpsl_obj
-- FOR EACH ROW
-- EXECUTE FUNCTION trg_delete_rpsl_obj();
-----------------------------------------------------------------------------------------------


------------------------------------------------------------------
--check mntner_obj before insert maintainer 
CREATE OR REPLACE FUNCTION check_mnter_obj_before_insert_mnt()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM maintainer WHERE mntner_name = NEW.mntner_name) THEN
    INSERT INTO maintainer (mntner_name) VALUES (NEW.mntner_name);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_mntner_obj_mnt
BEFORE INSERT ON mntner_obj
FOR EACH ROW
EXECUTE FUNCTION check_mnter_obj_before_insert_mnt();
--*********************************************************
--check rpsl_obj_mnt_by before insert maintainer 
CREATE OR REPLACE FUNCTION check_rpsl_obj_mnt_by_before_insert_mnt()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM maintainer WHERE mntner_name = NEW.mntner_name) THEN
    INSERT INTO maintainer (mntner_name) VALUES (NEW.mntner_name);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_rpsl_obj_mnt_by_mnt
BEFORE INSERT ON rpsl_obj_mnt_by
FOR EACH ROW
EXECUTE FUNCTION check_rpsl_obj_mnt_by_before_insert_mnt();

--***************************************************************
--check mbrs_by_ref before insert maintainer 
CREATE OR REPLACE FUNCTION check_mbrs_by_ref_before_insert_mnt()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM maintainer WHERE mntner_name = NEW.mntner_name) THEN
    INSERT INTO maintainer (mntner_name) VALUES (NEW.mntner_name);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_mbrs_by_ref_mnt
BEFORE INSERT ON mbrs_by_ref
FOR EACH ROW
EXECUTE FUNCTION check_mbrs_by_ref_before_insert_mnt();

-------------------------------------------------------------------------

--check mbrs_by_ref before insert autonomous_system 
CREATE OR REPLACE FUNCTION check_aut_num_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.as_num) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.as_num);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER trigger_before_insert_aut_num_autosys
BEFORE INSERT ON aut_num
FOR EACH ROW
EXECUTE FUNCTION check_aut_num_before_insert_autosys();
--********************************************************
--check exchange_report before insert autonomous_system
CREATE OR REPLACE FUNCTION check_exrpt_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.from_as) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.from_as);
  END IF;
  
   IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.to_as) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.to_as);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_exchange_report_autosys
BEFORE INSERT ON exchange_report
FOR EACH ROW
EXECUTE FUNCTION check_exrpt_before_insert_autosys();
--************************************************************
--check provide_customer before insert autonomous system
CREATE OR REPLACE FUNCTION check_prdcst_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.provider) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.provider);
  END IF;
  
   IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.customer) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.customer);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_provide_customer_autosys
BEFORE INSERT ON provide_customer
FOR EACH ROW
EXECUTE FUNCTION check_prdcst_before_insert_autosys();
--**************************************************************
--check route_obj before insert autonomous_system
CREATE OR REPLACE FUNCTION check_aut_num_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.as_num) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.as_num);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_route_obj_autosys
BEFORE INSERT ON route_obj
FOR EACH ROW
EXECUTE FUNCTION check_aut_num_before_insert_autosys();
--****************************************************************
--appied in peer
CREATE OR REPLACE FUNCTION check_peer_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.peer_1) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.peer_1);
  END IF;
  
   IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.peer_2) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.peer_2);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER trigger_before_insert_peer
BEFORE INSERT ON peer
FOR EACH ROW
EXECUTE FUNCTION check_peer_before_insert_autosys();


--*******************************************************************
--check as_set_contains_num before insert autonomous_system
CREATE OR REPLACE FUNCTION check_ascnum_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.as_num) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.as_num);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_as_set_contains_num
BEFORE INSERT ON as_set_contains_num
FOR EACH ROW
EXECUTE FUNCTION check_ascnum_before_insert_autosys();

--*****************************************************************
CREATE OR REPLACE FUNCTION check_rscnum_before_insert_autosys()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM autonomous_system WHERE as_num = NEW.as_num) THEN
    INSERT INTO autonomous_system (as_num) VALUES (NEW.as_num);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_insert_route_set_contains_num
BEFORE INSERT ON route_set_contains_num
FOR EACH ROW
EXECUTE FUNCTION check_rscnum_before_insert_autosys();

