import storyestimates
import requests
import pytest
import random
import string

def _random_string(size):
    return ''.join(random.choice(string.ascii_uppercase + string.digits) for _ in range(size))

def _make_up_user():
    return (_random_string(6), _random_string(16))

def _extract_user(session_data, nickname):
    users = session_data['users']
    for user_data in users:
        if user_data['nickname'] == nickname:
            return user_data

def _create_active_session(url, port):
    bob = storyestimates.StoryClient(url, port)
    bob.create_user()
    bob.create_session()
    bob.join_session('bob')

    bill = storyestimates.StoryClient(url, port)
    bill.create_user()
    bill.session = bob.session
    bill.join_session('bill')

    return (bob, bill)

def test_create_user(url, port):
    sec = storyestimates.StoryClient(url, port)
    user_id, user_token = sec.create_user()
    assert user_id
    assert user_token

def test_create_check_user(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.check_user()

def test_create_check_no_user(url, port):
    sec = storyestimates.StoryClient(url, port)
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.check_user()
    assert excinfo.value.response.status_code == 401

def test_create_check_bad_user(url, port):
    sec = storyestimates.StoryClient(url, port)
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.check_user()
    assert excinfo.value.response.status_code == 401

def test_create_session(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    session_data = sec.create_session()
    assert session_data['state'] == "Clean"
    assert session_data['users'] == []
    assert session_data['average'] is None
    assert session_data['session_id']

def test_create_session_no_auth(url, port):
    sec = storyestimates.StoryClient(url, port)
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.create_session()
    assert excinfo.value.response.status_code == 401

def test_join_session(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()
    sec.join_session('bob')

def test_lookup_session(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    create_data = sec.create_session()
    lookup_data = sec.lookup_session()
    assert create_data == lookup_data
    assert lookup_data['session_id'] == sec.session

def test_lookup_session_with_user(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()
    sec.join_session('bob')
    lookup_data = sec.lookup_session()
    assert lookup_data['users'][0]['nickname'] == 'bob'

def test_lookup_missing_session(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.session = _random_string(16)
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.lookup_session()
    assert excinfo.value.response.status_code == 404

def test_delete_session(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()
    sec.delete_session()
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.lookup_session()
    assert excinfo.value.response.status_code == 404

def test_delete_missing_session(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.session = _random_string(16)
    sec.create_user()
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.delete_session()
    assert excinfo.value.response.status_code == 404

def test_delete_session_no_auth(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()
    sec.auth = None
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.delete_session()
    assert excinfo.value.response.status_code == 401

def test_delete_session_bad_auth(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()
    sec.create_user()
    with pytest.raises(requests.HTTPError) as excinfo:
        sec.delete_session()
    assert excinfo.value.response.status_code == 403

def test_change_nickname(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()

    sec.join_session('bob')
    lookup_data = sec.lookup_session()
    assert len(lookup_data['users']) == 1
    assert lookup_data['users'][0]['nickname'] == 'bob'

    sec.join_session('bill')
    lookup_data = sec.lookup_session()
    assert len(lookup_data['users']) == 1
    assert lookup_data['users'][0]['nickname'] == 'bill'

def test_place_vote(url, port):
    sec = storyestimates.StoryClient(url, port)
    sec.create_user()
    sec.create_session()

    sec.join_session('bob')
    lookup_data = sec.lookup_session()
    assert len(lookup_data['users']) == 1
    assert lookup_data['users'][0]['vote_amount'] is None
    assert lookup_data['users'][0]['vote_state'] == "Empty"

    sec.place_vote(5)
    lookup_data = sec.lookup_session()
    assert len(lookup_data['users']) == 1
    assert lookup_data['users'][0]['vote_amount'] is None
    assert lookup_data['users'][0]['vote_state'] == "Hidden"

def test_collect_vote(url, port):
    bob, bill = _create_active_session(url, port)
    bob.place_vote(5)
    bob.reveal_votes()
    lookup_data = bob.lookup_session()

    assert len(lookup_data['users']) == 2
    bob_data = _extract_user(lookup_data, 'bob')
    bill_data = _extract_user(lookup_data, 'bill')

    assert bob_data['vote_amount'] == 5
    assert bob_data['vote_state'] == 'Visible'

    assert bill_data['vote_amount'] is None
    assert bill_data['vote_state'] == 'Empty'

    assert lookup_data['average'] == 5

def test_repeat_vote(url, port):
    bob, bill = _create_active_session(url, port)
    bob.place_vote(5)
    bob.reveal_votes()
    bill.place_vote(8)
    bob.repeat_votes()
    lookup_data = bob.lookup_session()

    assert len(lookup_data['users']) == 2
    bob_data = _extract_user(lookup_data, 'bob')
    bill_data = _extract_user(lookup_data, 'bill')

    assert bob_data['vote_amount'] is None
    assert bob_data['vote_state'] == 'Empty'

    assert bill_data['vote_amount'] is None
    assert bill_data['vote_state'] == 'Hidden'

def test_reset_vote(url, port):
    bob, bill = _create_active_session(url, port)
    bob.place_vote(5)
    bob.reveal_votes()
    bill.place_vote(8)
    bob.reset_votes()
    lookup_data = bob.lookup_session()

    assert len(lookup_data['users']) == 2
    bob_data = _extract_user(lookup_data, 'bob')
    bill_data = _extract_user(lookup_data, 'bill')

    assert bob_data['vote_amount'] is None
    assert bob_data['vote_state'] == 'Empty'

    assert bill_data['vote_amount'] is None
    assert bill_data['vote_state'] == 'Empty'

def test_average_vote(url, port):
    bob, bill = _create_active_session(url, port)
    bob.place_vote(5)
    bill.place_vote(7)
    bob.reveal_votes()
    lookup_data = bob.lookup_session()

    assert len(lookup_data['users']) == 2
    bob_data = _extract_user(lookup_data, 'bob')
    bill_data = _extract_user(lookup_data, 'bill')

    assert bob_data['vote_amount'] == 5
    assert bob_data['vote_state'] == 'Visible'

    assert bill_data['vote_amount'] == 7
    assert bill_data['vote_state'] == 'Visible'

    assert lookup_data['average'] == 6

def test_set_state_dirty(url, port):
    bob, bill = _create_active_session(url, port)
    with pytest.raises(requests.HTTPError) as excinfo:
        bob._update_session("Dirty")
    assert excinfo.value.response.status_code == 400

def test_set_state_other(url, port):
    bob, bill = _create_active_session(url, port)
    with pytest.raises(requests.HTTPError) as excinfo:
        bob._update_session("Somethingtotallynotastate")
    assert excinfo.value.response.status_code == 400

def test_non_admin_leave_session(url, port):
    bob, bill = _create_active_session(url, port)
    bill.leave_session()
    lookup_data = bob.lookup_session()
    assert _extract_user(lookup_data, 'bob')
    assert _extract_user(lookup_data, 'bill') is None

def test_admin_kick_user(url, port):
    bob, bill = _create_active_session(url, port)
    bob.leave_session(user_id=bill.user_id)
    lookup_data = bob.lookup_session()
    assert _extract_user(lookup_data, 'bob')
    assert _extract_user(lookup_data, 'bill') is None

def test_admin_kick_non_existent_user(url, port):
    bob, bill = _create_active_session(url, port)
    with pytest.raises(requests.HTTPError) as excinfo:
        bob.leave_session(user_id=_random_string(6))
    assert excinfo.value.response.status_code == 404

def test_non_admin_kick_user(url, port):
    bob, bill = _create_active_session(url, port)

    with pytest.raises(requests.HTTPError) as excinfo:
        bill.leave_session(user_id=bob.user_id)
    assert excinfo.value.response.status_code == 403

def test_grant_admin(url, port):
    bob, bill = _create_active_session(url, port)

    lookup_data = bob.lookup_session()
    assert bill.user_id not in lookup_data['admins']

    bob.grant_admin(bill.user_id)
    lookup_data = bob.lookup_session()
    assert bill.user_id in lookup_data['admins']

def test_revoke_admin(url, port):
    bob, bill = _create_active_session(url, port)

    lookup_data = bob.lookup_session()
    assert bill.user_id not in lookup_data['admins']

    bob.grant_admin(bill.user_id)
    lookup_data = bob.lookup_session()
    assert bill.user_id in lookup_data['admins']

    bob.revoke_admin(bill.user_id)
    lookup_data = bob.lookup_session()
    assert bill.user_id not in lookup_data['admins']

def test_non_admin_grant_admin(url, port):
    bob, bill = _create_active_session(url, port)

    with pytest.raises(requests.HTTPError) as excinfo:
        bill.grant_admin(bill.user_id)
    assert excinfo.value.response.status_code == 403

def test_non_admin_revoke_admin(url, port):
    bob, bill = _create_active_session(url, port)

    with pytest.raises(requests.HTTPError) as excinfo:
        bill.revoke_admin(bob.user_id)
    assert excinfo.value.response.status_code == 403

def test_revoke_self_admin(url, port):
    bob, bill = _create_active_session(url, port)

    lookup_data = bob.lookup_session()
    assert bob.user_id in lookup_data['admins']
    assert bill.user_id not in lookup_data['admins']

    bob.revoke_admin(bob.user_id)
    lookup_data = bob.lookup_session()
    assert bob.user_id not in lookup_data['admins']
    assert bill.user_id not in lookup_data['admins']

def test_grant_admin_to_admin(url, port):
    bob, bill = _create_active_session(url, port)

    lookup_data = bob.lookup_session()
    assert bill.user_id not in lookup_data['admins']

    bob.grant_admin(bill.user_id)
    lookup_data = bob.lookup_session()
    assert bill.user_id in lookup_data['admins']

    with pytest.raises(requests.HTTPError) as excinfo:
        bob.grant_admin(bill.user_id)
    assert excinfo.value.response.status_code == 400

def test_revoke_admin_from_non_admin(url, port):
    bob, bill = _create_active_session(url, port)

    lookup_data = bob.lookup_session()
    assert bill.user_id not in lookup_data['admins']

    with pytest.raises(requests.HTTPError) as excinfo:
        bob.revoke_admin(bill.user_id)
    assert excinfo.value.response.status_code == 400
