import requests
import json


class StoryClient:
    def __init__(self, url, port):
        self.url = url
        self.port = port
        self.auth = None
        self.session = None

    def _make_url(self, path):
        return "{}:{}/{}".format(self.url, self.port, path)

    @property
    def user_id(self):
        if self.auth is not None:
            uid, _ = self.auth
            return uid
        else:
            return None

    def _do_patch(self, path, body):
        url = self._make_url(path)

        response = requests.patch(
            url,
            json=body,
            auth=self.auth
        )
        response.raise_for_status()
        return response.json()

    def _do_put(self, path, body):
        url = self._make_url(path)

        response = requests.put(
            url,
            json=body,
            auth=self.auth
        )
        response.raise_for_status()
        return response.json()

    def _do_post(self, path, body):
        url = self._make_url(path)

        response = requests.post(
            url,
            json=body,
            auth=self.auth
        )
        response.raise_for_status()
        return response.json()

    def _do_delete(self, path):
        url = self._make_url(path)

        response = requests.delete(
            url,
            auth=self.auth
        )
        response.raise_for_status()

    def _do_get(self, path, params):
        url = self._make_url(path)

        response = requests.get(
            url,
            params=params,
            auth=self.auth
        )
        response.raise_for_status()
        return response.json()

    def create_user(self):
        user_data = self._do_post('api/user', {})
        user_token = user_data['user_token']
        user_id = user_data['user_id']
        self.auth = (user_id, user_token)
        return (user_id, user_token)

    def check_user(self):
        self._do_get('api/user', {})

    def lookup_session(self):
        return self._do_get('api/session/{}'.format(self.session), {})

    def delete_session(self):
        return self._do_delete('api/session/{}'.format(self.session))

    def leave_session(self, user_id=None):
        if user_id is None:
            user_id = self.user_id
        return self._do_delete('api/session/{}/user/{}'.format(self.session, user_id))

    def create_session(self):
        session_data = self._do_post('api/session', {})
        self.session = session_data['session_id']
        return session_data

    def join_session(self, nickname):
        url = 'api/session/{}/user/{}'.format(self.session, self.user_id)
        body = {'nickname': nickname}
        self._do_put(url, body)

    def place_vote(self, vote):
        url = 'api/session/{}/user/{}/vote'.format(self.session, self.user_id)
        body = {'vote': vote}
        self._do_post(url, body)

    def _update_session(self, state):
        url = 'api/session/{}'.format(self.session)
        body = {'state': state}
        self._do_patch(url, body)

    def reveal_votes(self):
        self._update_session("Visible")

    def repeat_votes(self):
        self._update_session("Voting")

    def reset_votes(self):
        self._update_session("Clean")

    def grant_admin(self, user_id):
        self._do_post('api/session/{}/admin/{}'.format(self.session, user_id), {})

    def revoke_admin(self, user_id):
        self._do_delete('api/session/{}/admin/{}'.format(self.session, user_id))
